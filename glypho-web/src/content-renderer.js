class ContentRenderer extends HTMLElement {
  static get observedAttributes() {
    return ["content"];
  }

  constructor() {
    console.log("constructer");
    super();
  }

  connectedCallback() {
    this._update();
  }

  disconnectedCallback() {
    // Clean up pending updates
    if (this._updateTimeout) {
      clearTimeout(this._updateTimeout);
      this._updateTimeout = null;
    }
  }

  /**
   * Called when observed attributes change (following Datastar pattern)
   */
  attributeChangedCallback(name, oldValue, newValue) {
    console.log(name);
    const value = `You entered: ${newValue}`;
    console.log("updated shit");
  }

  connectedMoveCallback() {
    console.log("Custom move-handling logic here.");
    this.forceUpdate();
  }
  /**
   * Schedule an update with debouncing to avoid multiple rapid updates
   */
  _scheduleUpdate() {
    if (this._updateTimeout) {
      clearTimeout(this._updateTimeout);
    }

    this._updateTimeout = setTimeout(() => {
      this._update();
      this._updateTimeout = null;
    }, 100); // 100ms debounce
  }

  /**
   * Update both Prism and MathJax rendering
   */
  async _update() {
    this._updatePrism();
    await this._updateMathJax();

    // Dispatch custom event when rendering is complete (props down, events up)
    console.log("ALGOOO");
    this.dispatchEvent(
      new CustomEvent("rendered", {
        detail: {
          timestamp: Date.now(),
          hasContent: this.innerHTML.trim().length > 0,
        },
        bubbles: true,
        composed: true,
      }),
    );
  }

  /**
   * Update Prism syntax highlighting
   */
  _updatePrism() {
    if (typeof Prism !== "undefined" && Prism.highlightAllUnder) {
      Prism.highlightAllUnder(this);
    }
  }
  /**
   * Force an immediate update of rendering
   */
  forceUpdate() {
    if (this._updateTimeout) {
      clearTimeout(this._updateTimeout);
      this._updateTimeout = null;
    }
    this._update();
  }
}

// Register the custom element
customElements.define("content-renderer", ContentRenderer);
