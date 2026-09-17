" nvim-plugin/glypho-nvim/plugin/glypho.vim
" Neovim plugin for running the glypho tool

" Define the :Glypho command
" It takes one argument: the path to the markdown file
command! -nargs=? Glypho call v:lua.require('glypho')._run_glypho_cmd_handler(expand('<f-args>'))

" Optional: Provide an example configuration for the user
" In their init.lua, users can set:
" vim.g.glypho_executable_path = '~/path/to/your/glypho/executable'
" Or, if glypho is in their PATH, no configuration is needed.