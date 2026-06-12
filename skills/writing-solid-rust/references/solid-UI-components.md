# Domain: UI component port

You can structure UI components in Rust with a SOLID-ish, hexagonal flavor by treating “rendering backends” as adapters behind a trait-based UI port. [web.mit](https://web.mit.edu/rust-lang_v1.25/arch/amd64_ubuntu1404/share/doc/rust/html/book/second-edition/ch17-02-trait-objects.html)

```rust
// ui/domain.rs

pub struct Size {
    pub width: u16,
    pub height: u16,
}

pub struct Position {
    pub x: u16,
    pub y: u16,
}

/// Render-agnostic description of what to draw.
pub enum UiNode {
    Text { pos: Position, content: String },
    Button { pos: Position, label: String },
    Container { children: Vec<UiNode> },
}

/// Port: a render-agnostic UI component.
pub trait Component {
    fn render(&self) -> UiNode;
}
```

## Domain components

```rust
// ui/components.rs

use crate::ui::domain::{Component, Position, UiNode};

pub struct Label {
    pub pos: Position,
    pub text: String,
}

impl Component for Label {
    fn render(&self) -> UiNode {
        UiNode::Text {
            pos: Position { x: self.pos.x, y: self.pos.y },
            content: self.text.clone(),
        }
    }
}

pub struct Button {
    pub pos: Position,
    pub label: String,
}

impl Component for Button {
    fn render(&self) -> UiNode {
        UiNode::Button {
            pos: Position { x: self.pos.x, y: self.pos.y },
            label: self.label.clone(),
        }
    }
}

pub struct LoginForm<L: Component, B: Component> {
    pub title: L,
    pub submit: B,
}

impl<L: Component, B: Component> Component for LoginForm<L, B> {
    fn render(&self) -> UiNode {
        UiNode::Container {
            children: vec![self.title.render(), self.submit.render()],
        }
    }
}
```

## Rendering ports and adapters

```rust
// ui/render.rs

use crate::ui::domain::{UiNode, Position};

pub trait Renderer {
    fn draw_node(&mut self, node: &UiNode);
    fn flush(&mut self);
}
```

```rust
// infra/terminal_renderer.rs

use crate::ui::domain::{UiNode};
use crate::ui::render::Renderer;

pub struct TerminalRenderer;

impl Renderer for TerminalRenderer {
    fn draw_node(&mut self, node: &UiNode) {
        match node {
            UiNode::Text { pos, content } => {
                println!("(term) text @({},{}) -> {}", pos.x, pos.y, content);
            }
            UiNode::Button { pos, label } => {
                println!("(term) button @({},{}) [{}]", pos.x, pos.y, label);
            }
            UiNode::Container { children } => {
                for child in children {
                    self.draw_node(child);
                }
            }
        }
    }

    fn flush(&mut self) {
        // No-op for demo.
    }
}
```

```rust
// infra/web_renderer.rs

use crate::ui::domain::UiNode;
use crate::ui::render::Renderer;

pub struct WebRenderer {
    pub html: String,
}

impl WebRenderer {
    pub fn new() -> Self {
        Self { html: String::new() }
    }
}

impl Renderer for WebRenderer {
    fn draw_node(&mut self, node: &UiNode) {
        match node {
            UiNode::Text { content, .. } => {
                self.html.push_str(&format!("<p>{}</p>\n", content));
            }
            UiNode::Button { label, .. } => {
                self.html.push_str(&format!("<button>{}</button>\n", label));
            }
            UiNode::Container { children } => {
                self.html.push_str("<div class=\"container\">\n");
                for child in children {
                    self.draw_node(child);
                }
                self.html.push_str("</div>\n");
            }
        }
    }

    fn flush(&mut self) {
        // e.g., send HTML to client; here just log.
        println!("Generated HTML:\n{}", self.html);
    }
}
```

## Composition root

```rust
// main.rs

mod ui {
    pub mod domain;
    pub mod components;
    pub mod render;
}
mod infra {
    pub mod terminal_renderer;
    pub mod web_renderer;
}

use ui::components::{Label, Button, LoginForm};
use ui::domain::{Component, Position};
use ui::render::Renderer;
use infra::terminal_renderer::TerminalRenderer;
use infra::web_renderer::WebRenderer;

fn build_login_form() -> impl Component {
    let title = Label {
        pos: Position { x: 0, y: 0 },
        text: "Login".into(),
    };
    let submit = Button {
        pos: Position { x: 0, y: 1 },
        label: "Sign in".into(),
    };
    LoginForm { title, submit }
}

fn main() {
    let form = build_login_form();
    let tree = form.render();

    // Terminal adapter
    let mut term = TerminalRenderer;
    term.draw_node(&tree);
    term.flush();

    // Web adapter
    let mut web = WebRenderer::new();
    web.draw_node(&tree);
    web.flush();
}
```

## Where SOLID shows up

- **SRP**
  - `Label`, `Button`, and `LoginForm` each have a single reason to change (their own layout/structure).
  - Renderers only know how to turn `UiNode` into terminal output or HTML, not business/UI logic. [raphlinus.github](https://raphlinus.github.io/rust/gui/2022/05/07/ui-architecture.html)

- **OCP**
  - New component types: extend `UiNode` + `impl Component` without touching existing components.
  - New backends: add another `Renderer` impl (e.g., GPUI, Dioxus, rxtui) without changing components. [docs](https://docs.rs/rxtui/latest/rxtui/)

- **DIP**
  - Domain depends on the `Component` and `Renderer` traits, not on terminal/web frameworks.
  - `main` wires concrete components to concrete renderers (composition root). [github](https://github.com/angelocatalani/hexagonal_architecture)
