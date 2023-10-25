// Copyright 2023 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

/**
 * @fileoverview Implements the main UI for the board.
 */

// How many linkes to keep in the log.
const MAX_LOG_LINES = 300;

/** Websocket wrapper class.
 *
 * Ensures that a connection to the backend is healthy, or tries to fix it.
 * Serializes messages to and from the backend.
 *
 * @class MessageChannel
 */
class MessageChannel {
  constructor({urlEndpoint, onOpen, onMessage, onClose}) {
    this._args = {urlEndpoint, onOpen, onMessage, onClose};
    this._connect(this._args);
  }

  _connect({urlEndpoint, onOpen, onMessage, onClose}) {
    this.ws = new WebSocket(urlEndpoint);
    this.ws.onopen = (event) => onOpen();
    this.ws.onmessage = (event) => onMessage(this._parseMessage(event));
    this.ws.onclose = (event) => {
      setTimeout(() => this._connect(this._args), 1000);
      onClose();
    };
    this.ws.onerror = (event) => {
      this.ws.close();
    };
  }

  send(message) {
    this.ws.send(this._encodeMessage(message));
  }

  _encodeMessage(message) {
    return JSON.stringify(message);
  }

  _parseMessage(event) {
    const message = JSON.parse(event.data);
    return message;
  }
}

/** Draws the board.
 *
 * Since the app is simple, this draws the board directly as an SVG.
 * If the complexity of the UI increases, we'll move to a JS framework like
 * Vue or Lit.
 *
 * @class BoardDrawer
 */
class BoardDrawer {
  constructor(outputElement) {
    this.ioPanel = document.createElement("div");
    this.log = document.createElement("div");
    this.inputWrapper = document.createElement("div");
    this.input = document.createElement("input");
    this.button = document.createElement("button");
    this.board = document.createElement("div");
    this.overlay = document.createElement("div");
    this.overlayCentered = document.createElement("div");
    this.overlayCenteredTitle = document.createElement("div");
    this.overlayCenteredMessage = document.createElement("div");
    this.overlay.appendChild(this.overlayCentered);
    this.overlayCentered.appendChild(this.overlayCenteredTitle);
    this.overlayCentered.appendChild(this.overlayCenteredMessage);
    this.inputWrapper.appendChild(this.input);
    this.inputWrapper.appendChild(this.button);
    this.ioPanel.appendChild(this.log);
    this.ioPanel.appendChild(this.inputWrapper);
    outputElement.appendChild(this.ioPanel);
    outputElement.appendChild(this.board);
    outputElement.appendChild(this.overlay);
    this._svg = SVG().addTo(this.board).size("100%", "100%");
    this.setVisibility(false);
    this.reset();
  }

  /** Places a component on the board.
   *
   */
  async placeComponent({type}) {
    const response = await fetch(`components/${type}.svg`);
    const component = this._svg
      .group()
      .svg(await response.text())
      .move(10, this._nextComponentY);
    this._nextComponentY = component.height() + 10 + component.y();
    this._nextComponentX = Math.max(
      component.width() + component.x(),
      this._nextComponentX,
    );
    this.ioPanel.style.left = `${this._nextComponentX + 40}px`;
    return component;
  }

  async reset() {
    this._nextComponentY = 90;
    this._nextComponentX = 0;
    // Destroy log.
    this.log.textContent = "";
  }

  async start() {
    await this._build();
  }

  /** Saves the callback to call when the user types something.
   *
   */
  async setInputCallback(callback) {
    this._inputCallback = callback;
  }

  async _build() {
    this.overlay.style.position = "fixed";
    this.overlay.style.top = "0";
    this.overlay.style.bottom = "0";
    this.overlay.style.right = "0";
    this.overlay.style.left = "0";
    this.overlay.style.background = "white";
    this.overlay.style.display = "flex";
    this.overlay.style.justifyContent = "center";
    this.overlay.style.alignItems = "center";
    this.overlayCentered.style.display = "flex";
    this.overlayCentered.style.flexDirection = "column";
    this.overlayCentered.style.justifyContent = "center";
    this.overlayCentered.style.alignItems = "center";
    this.overlayCenteredTitle.style.fontFamily = "'Montserrat', sans-serif";
    this.overlayCenteredTitle.style.fontSize = "1.5rem";
    this.overlayCenteredTitle.textContent = "Wasefire";
    this.overlayCenteredMessage.style.minHeight = "1rem";
    this.overlayCenteredMessage.style.marginTop = "1rem";
    this.overlayCenteredMessage.style.fontFamily = "'Montserrat', sans-serif";
    this.overlayCenteredMessage.style.fontSize = ".9rem";
    this.overlayCenteredMessage.style.color = "rgba(100,100,100,1)";
    this.board.style.minHeight = "300px";
    this.board.style.minWidth = "100%";
    this._svg
      .rect("100%", "100%")
      .fill("rgb(51, 126, 58)")
      .move(0, 12)
      .stroke({color: "rgb(12, 74, 19)", width: 2})
      .radius(5);
    const response = await fetch("title.svg");
    const title = this._svg.svg(await response.text());
    this.ioPanel.style.position = "absolute";
    this.ioPanel.style.minWidth = "20px";
    this.ioPanel.style.minHeight = "20px";
    this.ioPanel.style.background = "rgba(45,45,45,.95)";
    this.ioPanel.style.boxShadow = "3px 3px 0 rgba(10,10,10,.95)";
    this.ioPanel.style.top = "100px";
    this.ioPanel.style.right = "20px";
    this.ioPanel.style.left = "20px";
    this.ioPanel.style.bottom = "20px";
    this.ioPanel.style.display = "flex";
    this.ioPanel.style.flexDirection = "column";
    this.log.style.flexGrow = "1";
    this.log.style.color = "white";
    this.log.style.paddingLeft = "1rem";
    this.log.style.overflowY = "auto";
    this.log.style.flexDirection = "column-reverse";
    this.log.style.display = "flex";
    this.log.style.fontFamily = "'Fira Code'";
    this.inputWrapper.style.display = "flex";
    this.input.style.flexGrow = "1";
    this.input.focus();
    this.input.onkeyup = (event) => {
      if (event.key != "Enter") return;
      this._sendInput();
    };
    this.button.textContent = "Send";
    this.button.style.cursor = "pointer";
    this.button.onclick = () => this._sendInput();
  }

  async _sendInput() {
    this.button.disabled = true;
    if (this._inputCallback) {
      await this._inputCallback({
        type: "input",
        message: this.input.value,
      });
      this.appendInput(this.input.value);
    }
    this.input.value = "";
    this.button.disabled = false;
  }

  _dropOldLogElements() {
    [...this.log.children]
      .slice(MAX_LOG_LINES)
      .map((c) => this.log.removeChild(c));
  }

  _appendLog(message) {
    const p = document.createElement("p");
    p.innerText = message;
    p.style.margin = "3px 0";
    p.style.transition = "opacity .1s linear";
    p.style.opacity = "0.5";
    this.log.prepend(p);
    setTimeout(() => {
      p.style.opacity = "1";
    }, 1);
    this._dropOldLogElements();
    return p;
  }

  appendInput(message) {
    const p = this._appendLog(message);
    p.style.marginLeft = "auto";
    p.style.marginRight = "1rem";
    p.style.color = "rgba(200,255,255,0.7)";
  }

  appendStatus(message) {
    const p = this._appendLog(message);
    p.style.color = "rgba(0,255,255,0.7)";
    this.overlayCenteredMessage.innerText = message;
  }

  appendEvent(message) {
    const p = this._appendLog(message);
    p.style.color = "rgba(0,100,255,0.7)";
  }

  appendLog(message) {
    this._appendLog(message);
  }

  setVisibility(visible) {
    const overlayVisibility = visible ? "hidden" : "visible";
    if (this.overlay.style.visibility == overlayVisibility) return;
    this.overlay.style.visibility = overlayVisibility;
    this.overlayCenteredTitle.classList.remove("loading-title");
    this.overlayCenteredMessage.classList.remove("loading-message");
    if (visible) return;
    setTimeout(() => {
      this.overlayCenteredTitle.classList.add("loading-title");
      this.overlayCenteredMessage.classList.add("loading-message");
    }, 50);
  }
}

/** Top controller class.
 *
 * Creates a board and a message channel. Processes user messages.
 *
 * @class Board
 */
class Board {
  constructor(urlEndpoint, outputElement) {
    this._connected = false;
    this._components = [];
    this._drawer = new BoardDrawer(outputElement);
    this._channel = new MessageChannel({
      urlEndpoint,
      onOpen: () => {
        this._connected = true;
        this._drawer.setVisibility(false);
        this._drawer.appendStatus(
          "Connection established! Waiting for runner configuration...",
        );
      },
      onClose: () => {
        if (this._connected) {
          this._drawer.appendStatus(
            "Backend disconnected. Waiting for it to restart...",
          );
          this._connected = false;
        }
      },
      onMessage: (message) => this.processMessage(message),
    });
    this._drawer.setInputCallback((message) => this._channel.send(message));
    this._ensureOnlyThisTabIsOpen();
  }

  getComponentFromId(id) {
    for (const component of this._components) {
      if (component.id == id) return component;
    }
  }

  async processMessage(message) {
    const messageType = message["type"];
    if (messageType == "board_config") {
      this._drawer.appendStatus("Board configuration received.");
      this._drawer.setVisibility(true);
      this._setupBoard(message["components"]);
    } else if (messageType == "set") {
      const component = this.getComponentFromId(message["componentId"]);
      if (component) component.set(message["state"]);
    } else if (messageType == "disconnected") {
      this._drawer.appendStatus(
        "Runner disconnected. Waiting for new connection...",
      );
      this._drawer.setVisibility(false);
    } else if (messageType == "connected") {
      this._drawer.appendStatus("Backend connected. Waiting for runner...");
      this._drawer.setVisibility(false);
    } else if (messageType == "log") {
      this._drawer.appendLog(message["message"]);
    } else {
      this._drawer.appendStatus(`Unknown message: ${JSON.stringify(message)}`);
    }
  }

  async _setupBoard(componentsSpecs) {
    this._components.map((c) => c.destroy());
    this._drawer.reset();
    this._components = [];

    for (const componentSpecs of componentsSpecs) {
      const componentType = componentSpecs["type"];
      const componentId = componentSpecs["id"];
      const componentClass = {
        monochrome_led: MonochromeLedComponent,
        button: ButtonComponent,
      }[componentType];
      const component = new componentClass(
        componentId,
        this._drawer,
        this._channel,
      );
      await component.place();
      await component.reset();
      this._components.push(component);
    }
    this._drawer.appendStatus("Board is ready!");
  }

  async start() {
    await this._drawer.start();
  }

  _ensureOnlyThisTabIsOpen() {
    // Allows only one tab to be open on this site.
    // This prevents messages from being lost.
    const broadcast = new BroadcastChannel('intertab');
    broadcast.onmessage = function(event) {
      if (event?.data?.message === 'TAKEOVER')  window.close();
    }
    broadcast.postMessage({ message: 'TAKEOVER' });
  }
}
