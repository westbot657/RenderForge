# renderforge.rs

A wrapper around egui, eframe, and glow to simplify gl systems in particular for instanced rendering and material management







# Dev

## required features
-[x] gl state manager
-[ ] instanted meshes
-[ ] immediate/batch renderer
-[ ] texture and texture atlas system that should both work in any context.
-[ ] shader system (probably just part of gl state)
-[ ] window manager system, allow multiple windows, don't tie resources to a "parent" window, windows should be treated as an extension not a base
-[ ] heirarchial render tree, if data ever needs to be shared, allow for Rc<RefCell<>>s
-[ ] built-in click event handler, automatically determine the right element to click
-[ ] text input system, mostly just forward key events but only after processing keybinds and other stuff
-[ ] keybinds system, allow complex binds that can be re-assigned and serialized
-[ ] debug renderers
-[ ] UI widget system
-[ ] post processing pipeline








