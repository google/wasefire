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
 * @fileoverview Implements the available board components.
 */

/**
 * Board component abstract class.
 *
 * @class BoardComponent
 */
class BoardComponent {
  /**
   @constructor
   @abstract
   */
  constructor(componentId, boardDrawer, channel) {
    if (new.target === BoardComponent)
      throw new Error("Abstract class cannot be instantiated.");
    this._drawer = boardDrawer;
    this._channel = channel;
    this.id = componentId;
  }

  destroy() {
    if (!this._element) return;
    this._element.svg("");
  }

  /**
   @abstract
   */
  reset() {
    throw new Error("Abstract method called.");
  }
}

/**
 * A clickable button.
 *
 * @class ButtonComponent
 * @extends {BoardComponent}
 */
class ButtonComponent extends BoardComponent {
  constructor(componentId, boardDrawer, channel) {
    super(componentId, boardDrawer, channel);
    this._status = false;
  }

  async place() {
    this._element = await this._drawer.placeComponent({type: "button"});
    this._element.on("mousedown", () => this.onPressed());
    this._element.on("mouseup", () => this.onReleased());
  }

  onPressed() {
    this._channel.send({
      componentId: this.id,
      componentType: "button",
      state: "pressed",
    });
    this.set(true);
  }
  onReleased() {
    this._channel.send({
      componentId: this.id,
      componentType: "button",
      state: "released",
    });
    this.set(false);
  }

  reset() {
    this.set(false);
  }

  get() {
    return this._status;
  }

  set(value) {
    this._status = value;
    const pressed = this._element.findOne("#pressed");
    const notPressed = this._element.findOne("#not_pressed");
    if (value) {
      pressed.show();
      notPressed.hide();
    } else {
      notPressed.show();
      pressed.hide();
    }
  }
}

/**
 * A blinky led.
 *
 * @class ButtonComponent
 * @extends {BoardComponent}
 */
class MonochromeLedComponent extends BoardComponent {
  constructor(componentId, boardDrawer, channel) {
    super(componentId, boardDrawer, channel);
    this._status = false;
  }

  async place() {
    this._element = await this._drawer.placeComponent({type: "monochrome_led"});
  }

  reset() {
    this.set(false);
  }

  get() {
    return this._status;
  }

  set(value) {
    this._status = value;
    const light = this._element.findOne("#on");
    if (value) {
      light.show();
    } else {
      light.hide();
    }
  }
}
