local M = {}

--- Finds the glypho executable path.
--- It first checks `vim.g.glypho_executable_path`.
--- If not set, it tries to find 'glypho' in the system PATH.
--- @return string|nil The path to the glypho executable, or nil if not found.
function M.find_glypho_executable()
	-- 1. Check user-defined global variable
	if vim.g.glypho_executable_path and vim.g.glypho_executable_path ~= "" then
		local user_path = vim.fn.expand(vim.g.glypho_executable_path)
		if vim.fn.filereadable(user_path) then
			return user_path
		else
			vim.notify(
				'Glypho: Configured executable path "' .. user_path .. '" is not a readable file.',
				vim.log.levels.WARN
			)
		end
	end

	-- 2. Try to find in system PATH
	local result = vim.fn.exepath("glypho")
	if result and result ~= "" then
		return result
	end

	-- Fallback: Use the known build path (only works for current project dev)
	-- This part is specifically for the developer of the glypho project.
	-- For external users, they should configure glypho_executable_path or have glypho in PATH.
	local dev_path = vim.fn.expand("~/myprojects/glypho/target/release/glypho")
	if vim.fn.filereadable(dev_path) then
		vim.notify(
			'Glypho: Using developer build path "'
				.. dev_path
				.. '". Consider adding glypho to your PATH or setting vim.g.glypho_executable_path.',
			vim.log.levels.INFO
		)
		return dev_path
	end

	vim.notify(
		"Glypho: Could not find glypho executable. Please ensure it is in your system PATH or set vim.g.glypho_executable_path.",
		vim.log.levels.ERROR
	)
	return nil
end

--- Runs the glypho tool with the given markdown file.
--- @param markdown_file string The path to the markdown file.
function M.run_glypho(arg_file)
    local target_file = arg_file
    if not target_file or target_file == '' then
        target_file = vim.api.nvim_buf_get_name(0) -- Get path of current buffer
    end

    if not target_file or target_file == '' then
        vim.notify('Glypho: No markdown file provided and current buffer is not associated with a file.', vim.log.levels.ERROR)
        return
    end

    -- Basic check for markdown filetype/extension
    local file_ext = string.lower(vim.fn.fnamemodify(target_file, ':e'))
    local file_type = vim.bo.filetype -- Current buffer's filetype

    -- Allow if extension is markdown, md, or current buffer's filetype is markdown
    if not (file_ext == 'md' or file_ext == 'markdown' or file_type == 'markdown') then
        vim.notify('Glypho: Not a markdown file. File: ' .. target_file .. ' Ext: .' .. file_ext .. ' Filetype: ' .. file_type, vim.log.levels.WARN)
        return
    end

    local glypho_exec = M.find_glypho_executable()

    if not glypho_exec then
        return
    end

    local cmd = {glypho_exec, target_file}
    local escaped_cmd = vim.fn.join(vim.fn.map(cmd, 'shellescape(v:val)'))

    vim.notify('Running glypho: ' .. escaped_cmd, vim.log.levels.INFO)

    -- Use `termopen` to open a terminal buffer and run the command
    vim.api.nvim_command('split term://' .. escaped_cmd)
end

--- Handles the Neovim :Glypho command arguments from Vimscript.
--- It expands the argument and then calls M.run_glypho.
--- @param cmd_arg string Raw argument string from the command line.
function M._run_glypho_cmd_handler(expanded_arg)
    -- expanded_arg is already expanded by Vimscript
    if expanded_arg == '' then
        -- No argument provided, let M.run_glypho figure out the current buffer
        M.run_glypho(nil)
    else
        -- Argument provided, pass it to M.run_glypho
        M.run_glypho(expanded_arg)
    end
end

return M