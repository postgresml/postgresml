# pgml-components

`pgml-components` is a CLI for working with Rust web apps written with Rocket, Sailfish and SQLx, our toolkit of choice. It's currently a work in progress and only used internally by us, but the long term goal is to make it into a comprehensive framework for building web apps in Rust.

## Installation

`pgml-components` is available on crates.io and can be installed with `cargo install cargo-pgml-components`.

## Usage

To get a list of available commands:

```bash
cargo pgml-components --help
```

The CLI operates on a project directory, which is a directory containing a `Cargo.toml` file. You can specify the project directory with the `--project-path` flag, or you can run the CLI from the project directory itself.

### Commands

#### `bundle`

```bash
cargo pgml-components bundle
```

This command will read all the JavaScript and Sass files in the project and bundle them into a JS bundle and a CSS bundle accordingly. The JS bundle is created with [Rollup](https://rollupjs.org/) and the CSS bundle is created with the [Sass compiler](https://sass-lang.com/install/).

The `bundle` command should be ran after making any changes to JavaScript or Sass files. In our app, we added it to `build.rs` and run it on every change to the `src/` directory, but another way of running it without having to rebuild the app can be with `watch`:

```bash
cargo watch \
	--exec 'pgml-components bundle' \
	--watch src/ \
	--watch static/ \
	--ignore bundle.*.*
```

The bundles are placed in `static/css/style.css` and `static/js/bundle.js`. Both bundles are also copied into files with a short hash of their contents appended to their names, e.g. `static/css/style.6c1a4abc.css`. The bundles with the hash in their names are used in production, while the bundles without the hash are used in development. The hash is used to bust our caching of assets.

#### `add`

This command is used to add elements to the project. Currently, only frontend components are supported. Support for SQLx models and Rocket controllers is on the roadmap.

##### `add component`

```bash
cargo pgml-components add component <path>
```

This command will create a new frontend component in the specified path. The name of the component will be the absolute name of the Rust module. For example, if the path of the component is `dropdown`, then the component will be added to `src/components/dropdown` and it's name will be `crate::components::Dropdown`. If the component path is `controls/button/primary`, then component name will be `crate::components::controls::button::Primary` and the component will be placed into the `src/components/controls/button/primary` directory.

Frontend components use Sailfish templates, Hotwired Stimulus for JavaScript, and Sass stylesheets. The command creates all of these automatically and links both the JS and the Sass into the bundles produced by the `bundle` command.

For example, if creating the `dropdown` component, you'll get the following files:

```
# Sailfish template
src/components/dropdown/template.html

# Stimulus controller
src/components/dropdown/dropdown_controller.js

# Sass stylesheet
src/components/dropdown/dropdown.sass

# Rust module
src/components/dropdown/mod.rs
```

Initially, the component will be very barebones, but it will have all the necessary dependencies connected automatically.

###### `template.html`

The HTML template will just have a `<div>` that's connected to the Stimulus controller.

```html
<div data-controller="dropdown">
</div>
``` 

###### `dropdown_controller.js`

The Stimulus controller is connected to the `<div>` in the template above, and can be used immediately.

```javascript
import { Controller } from '@hotwired/stimulus'

export default class extends Controller {
	initiliaze() {
		console.log('Initialized dropdown controller')
	}
}
```

###### `dropdown.sass`

The Sass stylesheet doesn't have much, but you can start adding styles into it immediately. We don't have to use `data-controller` CSS selectors, the typical class selectors are fine. The command just generates something that will immediately work without any further configuration.

```css
div[data-controller="dropdown"] {
	width: 100%;
	height: 100px;

	background: red;
}
```

###### `mod.rs`

Everything is linked together ultimately with Rust. This file defines a struct that implements `sailfish::TemplateOnce`.

```rust
use sailfish::TemplateOnce;

#[derive(TemplateOnce)]
#[template(path = "dropdown/template.html")]
pub struct Dropdown {
	pub value: String,
}
```

Once the component is created, it can be used in any Sailfish template:

```html
<% use crate::components::Dropdown; %>

<div class="row">
	<div class="col-6">
		<%+ Dropdown::new() %>
	</div>
</div>
```

Components can be placed into any directory under `src/components`. They have to be in their own folder, so to have components organized neatly, you'd need to have folders that only contain other components and not be a component by itself.

For example, all buttons can be placed in `controls/buttons`, e.g. `controls/buttons/primary`, `controls/buttons/secondary`, but there cannot be a component in `controls/buttons`. There is no inherent limitation in our framework, but it's good to keep things tidy.

`pgml-components` does all of this automatically and makes sure that you don't accidently add components into a directory that already has one.

##### Deleting a component

There is no command for deleting a component yet, but you can do so by just deleting its directory and all the files in it, and bundling.

For example, to delete the `dropdown` component, you'd run:

```bash
rm -r src/components/dropdown
cargo pgml-components bundle
```

## Philosophy

`pgml-components` is an opinionated framework for building web apps in Rust based on our experience of using Rocket, SQLx, and Sailfish (and other template engines). That being said, its philosophy is to generate code based on its own templates, and doesn't force its user to use any specific parts of it. Therefore, all elements generated by `pgml-components` are optional. When creating a new component, you can remove the Stimulus controller or the Sass stylesheet. If removed, they won't be added into the bundle.
