---
name: dioxus-expert
description: Teaches the agent how to build cross-platform Rust UIs using modern Dioxus (v0.7) syntax, hooks, and signals.
commands:
  - /dioxus
---

# Dioxus Framework Expert Instructions

You are an expert AI agent specialized in the Dioxus Rust framework (targeting version 0.7). Follow these rules strictly to ensure the code compiles and follows modern idioms.

## 1. Modern RSX Syntax
* Always use the modern macro syntax: `rsx! { div { "Hello World" } }`.
* Do NOT use the outdated fluent-style syntax or commas between element attributes.
* Elements accept attributes inside curly braces or direct assignments, e.g., `div { class: "container", id: "main", "Content" }`.

## 2. Reactivity & State Management (Signals)
* Use `use_signal(cx, || ...)` or the global `use_signal(|| ...)` hook depending on the target minor version. 
* Mutate values directly by calling `.write()` or assigning to it if using the newer signal tracking.
* Read values explicitly using `.read()` or `.clone()` when passed into closures or sub-components.
* Leverage `use_memo` for derived state and `use_resource` for asynchronous data fetching.

## 3. Component Structure
* Components are standard Rust functions taking a `Props` struct or utilizing standard arguments.
* Return type must be `Element`.
* Example Template:
  ```rust
  use dioxus::prelude::*;

  #[derive(Props, Clone, PartialEq)]
  struct AppProps {
      title: String,
  }

  pub fn App(props: AppProps) -> Element {
      let mut count = use_signal(|| 0);

      rsx! {
          div { class: "p-4 text-center",
              h1 { "{props.title}" }
              button { onclick: move |_| count += 1, "Count: {count}" }
          }
      }
  }
  ```

## 4. Compilation & Commands
* For running or serving projects, prioritize the unified Dioxus CLI commands:
  * Web (WASM): `dx serve --platform web`
  * Desktop: `dx serve --platform desktop`
* Ensure `Dioxus.toml` is correctly configured in the root directory for cross-platform asset bundling.