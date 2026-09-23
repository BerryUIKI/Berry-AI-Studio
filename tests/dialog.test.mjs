import assert from "node:assert/strict";
import test from "node:test";

import {
  dialog,
  hasActiveDialog,
  getActiveDialogCount,
  getTopDialog,
  isEditableTarget,
  focusables,
  resetDialogStack,
} from "../src/utils/dialog.ts";

class MockElement {
  constructor(tagName = "div") {
    this.tagName = tagName.toUpperCase();
    this.attributes = new Map();
    this.children = [];
    this.parentElement = null;
    this.inert = false;
    this.tabIndex = 0;
    this.isConnected = true;
    this.id = "";
  }

  setAttribute(name, val) {
    this.attributes.set(name, String(val));
  }

  getAttribute(name) {
    return this.attributes.get(name) ?? null;
  }

  hasAttribute(name) {
    return this.attributes.has(name);
  }

  matches(sel) {
    if (sel.includes('[role="dialog"]') && this.getAttribute("role") === "dialog") return true;
    if (sel.includes('[role="alertdialog"]') && this.getAttribute("role") === "alertdialog") return true;
    return false;
  }

  querySelector(sel) {
    const tags = sel.split(",").map((s) => s.trim().toUpperCase());
    for (const child of this.children) {
      if (tags.includes(child.tagName)) return child;
      if (child.matches && child.matches(sel)) return child;
      const found = child.querySelector?.(sel);
      if (found) return found;
    }
    return null;
  }

  querySelectorAll(sel) {
    const list = [];
    for (const child of this.children) {
      if (child.matchesSelector && child.matchesSelector(sel)) {
        list.push(child);
      }
      if (child.querySelectorAll) {
        list.push(...child.querySelectorAll(sel));
      }
    }
    return list;
  }

  matchesSelector(sel) {
    if (sel.includes("button") && this.tagName === "BUTTON") return true;
    if (sel.includes("input") && this.tagName === "INPUT") return true;
    if (sel.includes("textarea") && this.tagName === "TEXTAREA") return true;
    if (sel.includes("select") && this.tagName === "SELECT") return true;
    if (sel.includes("[tabindex]") && this.tabIndex >= 0) return true;
    return false;
  }

  getClientRects() {
    return [{ width: 100, height: 100 }];
  }

  closest(sel) {
    if (sel.includes("input") && this.tagName === "INPUT") return this;
    if (sel.includes("textarea") && this.tagName === "TEXTAREA") return this;
    if (sel.includes("select") && this.tagName === "SELECT") return this;
    if (sel.includes('[contenteditable="true"]') && this.getAttribute("contenteditable") === "true") return this;
    if (sel.includes("[inert]") && this.inert) return this;
    return this.parentElement?.closest?.(sel) ?? null;
  }

  contains(node) {
    if (!node) return false;
    if (this === node) return true;
    for (const child of this.children) {
      if (child.contains && child.contains(node)) return true;
    }
    return false;
  }

  focus() {
    if (globalThis.document) {
      globalThis.document.activeElement = this;
    }
  }

  appendChild(child) {
    child.parentElement = this;
    this.children.push(child);
    return child;
  }
}

function setupDomMock() {
  const listeners = [];
  const root = new MockElement("body");

  globalThis.HTMLElement = MockElement;
  globalThis.document = {
    activeElement: null,
    body: root,
  };

  globalThis.window = {
    addEventListener(event, fn, capture) {
      listeners.push({ event, fn, capture });
    },
    removeEventListener(event, fn, capture) {
      const idx = listeners.findIndex((l) => l.event === event && l.fn === fn && l.capture === capture);
      if (idx >= 0) listeners.splice(idx, 1);
    },
    dispatchKeyEvent(event) {
      for (const l of [...listeners]) {
        if (l.event === "keydown") {
          l.fn(event);
        }
      }
    },
  };

  return { root, listeners };
}

function teardownDomMock() {
  resetDialogStack();
  delete globalThis.HTMLElement;
  delete globalThis.document;
  delete globalThis.window;
}

test("isEditableTarget correctly identifies inputs and contenteditables", () => {
  setupDomMock();
  try {
    const input = new MockElement("input");
    const textarea = new MockElement("textarea");
    const select = new MockElement("select");
    const div = new MockElement("div");
    const contentEditable = new MockElement("div");
    contentEditable.setAttribute("contenteditable", "true");

    assert.equal(isEditableTarget(input), true);
    assert.equal(isEditableTarget(textarea), true);
    assert.equal(isEditableTarget(select), true);
    assert.equal(isEditableTarget(contentEditable), true);
    assert.equal(isEditableTarget(div), false);
    assert.equal(isEditableTarget(null), false);
  } finally {
    teardownDomMock();
  }
});

test("dialog directive sets accessible role, modal attributes and titles", () => {
  setupDomMock();
  try {
    const overlay = new MockElement("div");
    const dialogPanel = new MockElement("div");
    dialogPanel.setAttribute("role", "dialog");
    const h2 = new MockElement("h2");
    dialogPanel.appendChild(h2);
    overlay.appendChild(dialogPanel);

    let closed = false;
    dialog.mounted(overlay, { value: () => { closed = true; } });

    assert.equal(hasActiveDialog(), true);
    assert.equal(getActiveDialogCount(), 1);
    assert.equal(dialogPanel.getAttribute("role"), "dialog");
    assert.equal(dialogPanel.getAttribute("aria-modal"), "true");
    assert.match(dialogPanel.getAttribute("aria-labelledby"), /^omera-dialog-/);
    assert.equal(h2.id, dialogPanel.getAttribute("aria-labelledby"));

    dialog.unmounted(overlay);
    assert.equal(hasActiveDialog(), false);
    assert.equal(getActiveDialogCount(), 0);
  } finally {
    teardownDomMock();
  }
});

test("dialog directive traps Tab and Shift+Tab focus navigation", () => {
  setupDomMock();
  try {
    const overlay = new MockElement("div");
    const btn1 = new MockElement("button");
    const btn2 = new MockElement("button");
    overlay.appendChild(btn1);
    overlay.appendChild(btn2);

    dialog.mounted(overlay, { value: () => {} });

    btn1.focus();
    assert.equal(globalThis.document.activeElement, btn1);

    // Tab from first to second is normal browser behavior, but Tab on last (btn2) wraps to first (btn1)
    btn2.focus();
    assert.equal(globalThis.document.activeElement, btn2);

    let defaultPrevented = false;
    globalThis.window.dispatchKeyEvent({
      key: "Tab",
      shiftKey: false,
      preventDefault() { defaultPrevented = true; },
    });
    assert.equal(defaultPrevented, true);
    assert.equal(globalThis.document.activeElement, btn1, "Tab from last button should wrap to first button");

    // Shift+Tab from first (btn1) wraps to last (btn2)
    defaultPrevented = false;
    globalThis.window.dispatchKeyEvent({
      key: "Tab",
      shiftKey: true,
      preventDefault() { defaultPrevented = true; },
    });
    assert.equal(defaultPrevented, true);
    assert.equal(globalThis.document.activeElement, btn2, "Shift+Tab from first button should wrap to last button");

    dialog.unmounted(overlay);
  } finally {
    teardownDomMock();
  }
});

test("dialog directive dismisses top dialog on Escape key and stops propagation", () => {
  setupDomMock();
  try {
    const overlay = new MockElement("div");
    let closeCalled = false;

    dialog.mounted(overlay, { value: () => { closeCalled = true; } });

    let prevented = false;
    let stopped = false;
    globalThis.window.dispatchKeyEvent({
      key: "Escape",
      preventDefault() { prevented = true; },
      stopImmediatePropagation() { stopped = true; },
    });

    assert.equal(closeCalled, true);
    assert.equal(prevented, true);
    assert.equal(stopped, true);

    dialog.unmounted(overlay);
  } finally {
    teardownDomMock();
  }
});

test("nested dialogs stack correctly and restore focus to opener", () => {
  const { root } = setupDomMock();
  try {
    const openerBtn = new MockElement("button");
    root.appendChild(openerBtn);
    openerBtn.focus();
    assert.equal(globalThis.document.activeElement, openerBtn);

    const dialog1 = new MockElement("div");
    const d1Btn = new MockElement("button");
    dialog1.appendChild(d1Btn);
    root.appendChild(dialog1);

    let d1Closed = false;
    dialog.mounted(dialog1, { value: () => { d1Closed = true; } });
    assert.equal(getActiveDialogCount(), 1);

    // Focus inside dialog 1
    d1Btn.focus();
    assert.equal(globalThis.document.activeElement, d1Btn);

    // Open nested dialog 2
    const dialog2 = new MockElement("div");
    const d2Btn = new MockElement("button");
    dialog2.appendChild(d2Btn);
    root.appendChild(dialog2);

    let d2Closed = false;
    dialog.mounted(dialog2, { value: () => { d2Closed = true; } });
    assert.equal(getActiveDialogCount(), 2);
    assert.equal(getTopDialog().element, dialog2);

    // Press Escape: should close dialog 2, not dialog 1
    globalThis.window.dispatchKeyEvent({
      key: "Escape",
      preventDefault() {},
      stopImmediatePropagation() {},
    });
    assert.equal(d2Closed, true, "Top dialog should close first");
    assert.equal(d1Closed, false, "Parent dialog should stay open");

    // Unmount dialog 2 (simulating Vue unmount after close)
    dialog.unmounted(dialog2);
    assert.equal(getActiveDialogCount(), 1);
    assert.equal(globalThis.document.activeElement, d1Btn, "Focus should return to previous active element in parent dialog");

    // Press Escape again: closes dialog 1
    globalThis.window.dispatchKeyEvent({
      key: "Escape",
      preventDefault() {},
      stopImmediatePropagation() {},
    });
    assert.equal(d1Closed, true);

    dialog.unmounted(dialog1);
    assert.equal(getActiveDialogCount(), 0);
    assert.equal(globalThis.document.activeElement, openerBtn, "Focus should return to initial opener button");
  } finally {
    teardownDomMock();
  }
});

test("background inertness isolates siblings while modal is open", () => {
  const { root } = setupDomMock();
  try {
    const mainGallery = new MockElement("main");
    const sidebar = new MockElement("aside");
    const modalContainer = new MockElement("div");
    root.appendChild(mainGallery);
    root.appendChild(sidebar);
    root.appendChild(modalContainer);

    assert.equal(mainGallery.inert, false);
    assert.equal(sidebar.inert, false);

    dialog.mounted(modalContainer, { value: () => {} });

    assert.equal(mainGallery.inert, true, "Gallery should become inert when modal mounts");
    assert.equal(sidebar.inert, true, "Sidebar should become inert when modal mounts");
    assert.equal(modalContainer.inert, false, "Modal itself must not become inert");

    dialog.unmounted(modalContainer);

    assert.equal(mainGallery.inert, false, "Gallery inert state should be restored on unmount");
    assert.equal(sidebar.inert, false, "Sidebar inert state should be restored on unmount");
  } finally {
    teardownDomMock();
  }
});

test("dialog listener teardown removes window event listener when stack empties", () => {
  const { listeners } = setupDomMock();
  try {
    const dlg1 = new MockElement("div");
    const dlg2 = new MockElement("div");

    assert.equal(listeners.length, 0);

    dialog.mounted(dlg1, { value: () => {} });
    assert.equal(listeners.length, 1);

    dialog.mounted(dlg2, { value: () => {} });
    assert.equal(listeners.length, 1, "Should not add duplicate global listeners for nested dialogs");

    dialog.unmounted(dlg2);
    assert.equal(listeners.length, 1, "Listener should remain while first dialog is still open");

    dialog.unmounted(dlg1);
    assert.equal(listeners.length, 0, "Global listener should be cleanly removed when all dialogs unmount");
  } finally {
    teardownDomMock();
  }
});
