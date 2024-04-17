---
description: How we are componentizing PostgresML with Rust, Rocket, Stimulus, and Sailfish.
tags: [engineering]
---

# Componentizing Rust

<div align="left">

<figure><img src=".gitbook/assets/daniel.jpg" alt="Author" width="125"><figcaption><p>Daniel Illenberger</p></figcaption></figure>

</div>

Daniel Illenberger

Apr 16, 2024

As a young physics student, I vividly remember my intro to programming with C course. A key concept my instructor Jack tried to impart on me was, do not repeat yourself.  The importance of this was lost on me with small one off projects, copy past was fast and if it worked it worked.  It was not until I began coding professionally that I really took the advice to heart.  

In theory we write simple pieces of code that can accomplish great things when combined.  In practice, especially in web development, original code can be faster in the moment and prevent the need for distilling out the common thread of functionality.  The lure is strong, but as coders of complex systems we must show restraint, else our projects become unmanageable and inconsistent. It is important to establish a practice of componentizing early on or we will be destined to a world of chaos. 

PostgresML is co-locating data and compute.  We put ML in your Postgres DB, this enhances speed and reduces the cost of doing ML in production.  As an ML DB company we understand the importance of reliable code.  This is why we chose Rust for all our projects, including our web app. Today I want to talk about how we use Rust, Rocket, Sailfish, and Stimulus to componentize our front end.  

## Follow Along
You can find all the code that goes along with this post [here](). 

PostgresML believes deeply in open source software and making AI available to all,  this is why we maintain our Postgres extension and dashboard freely available [here](https://github.com/postgresml/postgresml).  You can refer to it for more complex examples.   

## Setting up and getting started
I will leave it to the ready to follow the [Rust](https://www.rust-lang.org/learn/get-started) and [Rocket](https://rocket.rs/guide/v0.5/getting-started/) docs on getting a starter app running on there local machine.  If you have completed this successfully you should have a simple rocket application with the following file structure:  

- hello-component
	- src
	- main.rs
	- Cargo.toml

## Using Sailfish templates

Using templates to make components is the goto technique for web developers creating server side rendered apps, so I will just quickly run through getting started with sailfish.  If you need more, please visit [sailfish](https://rust-sailfish.github.io/sailfish/).  

Lets add Sailfish templates to this app. In the cargo.toml file add `sailfish = "0.8.0" # 0.8.1 has breaking changes` under dependencies. 

Tell sailfish where to find templates by adding a sailfish.toml file in the main directory, in it add `template_dirs = ["src/components"]`

Next we make a folder named components and in it make a new folder named layout.  This will be the base of our app that we can fill with other components. 
		
Each component will require a mod.rs and a template.html, lets make those now.  In layout/mod.rs place the following code. 

!!! code_block title="components/layout/mods.rs"
```rust 
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default, Clone)]
#[template(path = "layout/template.html")]
pub struct Layout {
  page: String,
}

impl Layout {
    pub fn new() -> Layout {
        Layout {
          page: String::new(),
        }
    }

    pub fn page(mut self, page: &str) -> Self {
        self.page = page.to_owned();
        self
    }
}

```
!!!

We will also need some html, create the following.

!!! code_block title="components/layout/template.html"
```html
&lt;!DOCTYPE html&gt;
&lt;html lang="en-US"&gt;
  &lt;head&gt;
    &lt;meta charset="UTF-8"&gt;
    &lt;title&gt;HTML Layout&lt;/title&gt;
  &lt;/head&gt;
  &lt;body&gt;
      &lt;main&gt;
        &lt;div&gt;
          &lt;%- page %&gt;
        &lt;/div&gt;
      &lt;/main&gt;

  &lt;/body&gt;
&lt;/html&gt;
```
!!!

Now lets make a second component called page.  This will also need a mod and template file:

!!! code_block title="components/page/mod.rs"
```rust
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default, Clone)]
#[template(path = "page/template.html")]
pub struct Page {
}

impl Page {
    pub fn new() -> Page {
        Page {}
    }
}
```
!!!

and the html:

!!! code_block title="components/pages/template.html"
```html
&lt;div&gt;Hello from page&lt;/div&gt;
```
!!!

we also need to make the components available so add the following to components/mod.rs

!!! code_block title="components/mod.rs"
```rust
// src/components/layout
pub mod layout;
pub use layout::Layout;

// src/components/page
pub mod page;
pub use page::Page;
```
!!!

!!! note

Current file structure

<ul>
  <li class="p-0 m-0">hello-component</li>
  <ul>
    <li class="p-0 m-0">src</li>
    <ul>
      <li class="p-0 m-0">components</li>
      <ul>
        <li class="p-0 m-0">layout</li>
        <ul>
          <li class="p-0 m-0">mode.rs</li>
          <li class="p-0 m-0">template.html</li>
        </ul>
      </ul>
      <ul>
        <li class="p-0 m-0">page</li>
        <ul>
          <li class="p-0 m-0">mode.rs</li>
          <li class="p-0 m-0">template.html</li>
        </ul>
      </ul>
    </ul>
    <li class="p-0 m-0">main.rs</li>
    <li class="p-0 m-0">Cargo.toml</li>
    <li class="p-0 m-0">sailfish.toml</li>
  </ul>
</ul>

!!!

Now we we have two components, a layout and a page.  We want our page to display inside our layout when the user visits our site. To do this we can set the page parameter of our layout struct to our page component using the setter function. 

!!! code_block title="main.rs"
```rust
#[get("/")]
fn index() -> ResponseOk {

    let page = Page::new().render_once().unwrap();
    let layout = Layout::new().page(&page);

    ResponseOk(layout.render_once().unwrap())
}
```
!!!

If we visit the "/" endpoint of our site we will see 

`Hello from page`

This is excellent, now we do not have to repeat ourselves when it comes to the head and body object of our html, we simply pass different pages to our layout.  For example, we can pass in a home page, about page, or contact page and never need to include the layout boiler plate in any of them.  We can add customization to our layout, like a custom head in the layout struct if needed. 

## Use of stimulus
This is a good start but any sufficiently complex site will eventually need JS. 

Stimulus is a JS framework used to tie bits of JS code to components.  It is an excellent tool for keeping your JS organized and simple.  We can add Stimulus to our projects by including the following code in our layout head:

!!! code_block title="components/layout/mod.rs"
```html
&lt;script defer src="https://unpkg.com/es-module-shims@1.9.0/dist/es-module-shims.js"&gt;&lt;/script&gt;
&lt;script type="importmap-shim">
  {
      "imports": {
          "@hotwired/stimulus": "https://unpkg.com/@hotwired/stimulus/dist/stimulus.js"
      }
  }
&lt;/script&gt;

&lt;script type="module-shim"&gt;
      import { Application } from '@hotwired/stimulus'
      import { default as Page } from '/static/js/page.js'

      const application = Application.start()
      application.register('page', Page)
&lt;/script&gt;
```
!!!

We will keep the JS files in a static directory, so lets add that in our project directory.  Your file structure should look like 
- hello-component
  - ...
  - static
    - js
      - page.js

Serve it by mounting a file server on the rocket instance with the mount method in main.rs `.mount("/static", rocket::fs::FileServer::from("static"))`

Now we can can add JS to our page component by including a data-controller=“js_component” tag.  And putting our JS in the js_component file.  Lets do that now and have our page greet us when we click a button.

in page.js lets put the following code:

!!! code_block title="static/js/page.js"
```javascript
import { Controller } from "@hotwired/stimulus"
static targets 

export default class extends Controller {
  connect() {
    console.log("Page controller is connected")
  }

  showGreating() {
    this.showMessage("Hello from page greeting")
  }

  showMessage(message) {
    this.messageTarget.textContent = message
  }
}
```
!!!

in page/template.html have the following 

!!! code_block title="components/page/template.html"
```html
&lt;div data-controller="page"&gt;
  &lt;button data-action="page#showGreeting"&gt;Greetings&lt;/button&gt;
  &lt;div data-page-target="message"&gt; &lt;/div&gt;
&lt;/div&gt;
```
!!!

if you restart your server and navigate to the home page we will see "Page controller is connected" in the console when the controller connects.  When we click our button we will see "Hello from page greeting" appear in the message area.

Of course we can simplify and keep all the JS and css in the component file if we use a bundler.  We have an open source tool that will do this for us [here](https://github.com/postgresml/postgresml/tree/master/packages/cargo-pgml-components)


## Inter-component communication
This is great, but we are not done yet.  To pull the most value from your code you will need to make these components communicate with each other after initial render.  It is easy to imagine an input element that needs to control external components, for example a copy button that informs the user it has copied content by flashing a toast.  The toast and the button should not be the same element since you may want to us the toast for other messages.  Also, we would not want to include multiple toast elements on the same page, so, we need our copy button to trigger a toast message. The correct solution per [stimulus](https://stimulus.hotwired.dev/reference/controllers#cross-controller-coordination-with-events) is to use events.

### Child to parent communication

Lets create a copy button that displays a message on our page. Create a new component and name it copy. In the new file create the following: 

!!! code_block title="components/copy/mod.rs"
```rust
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default, Clone)]
#[template(path = "copy/template.html")]
pub struct Copy {
  title: String,
  info: String,
}

impl Copy {
    pub fn new() -> Copy {
        Copy {
          title: "Copy".to_string(),
          info: "This is a copy component".to_string(),
        }
    }

    pub fn title(mut self, title: &str) -> Self {
        self.title = title.to_string();
        self
    }

    pub fn info(mut self, info: &str) -> Self {
        self.info = info.to_string();
        self
    }
}
```
!!!

Now make a template file with the following: 

!!! code_block title="components/copy/mod.rs"
```html
&lt;div data-controller="copy" style="display: flex; flex-direction: row; align-items: center; gap: 1rem;"&gt;
  &lt;p data-copy-target="info"&gt;&lt;%- info %&gt;&lt;/p&gt;
  &lt;button data-action="copy#copyItem" style="height: fit-content"&gt;
    &lt;%- title %&gt;
  &lt;/button&gt;
&lt;/div&gt;
```
!!!

and ensure the Copy component is available in the component mod.rs by adding 

!!! code_block title="components/mod.rs"
```rust
...

pub mod copy;
pub use copy::Copy;
```
!!!

now in page template use this component by adding `<% use crate::components::Copy; %>` at the beginning of the template on its own line and add a couple of these components in the main div.  It should look like this: 

!!! code_block title="components/page/template.html"
```html
&lt;% use crate::components::Copy; %&gt;

&lt;div data-controller="page" data-action="copy:showMessage-&gt;page#showMessageEvent"&gt;
  &lt;button data-action="page#showGreeting"&gt;Greetings&lt;/button&gt;
  &lt;%+ Copy::new().title("First Copy Button").info("First clipboard item") %&gt;
  &lt;%+ Copy::new().title("Second Copy Button").info("Second clipboard item") %&gt;
  &lt;div data-page-target="message"&gt;&lt;/div&gt;
&lt;/div&gt;
```
!!!

Now we need to make are Copy component controller.  In static/js make a file named copy and add 

!!! code_block title="static/js/copy.js"
```javascript
import { Controller } from "@hotwired/stimulus"

export default class extends Controller {
  static targets = [
    "info"
  ]

  connect() {
    console.log("Copy controller is connected")
  }

  showInfo() {
    this.dispatch("showMessage", {detail: {info: this.infoTarget.textContent}})
  }

  copyItem(){
    this.showInfo()
    navigator.clipboard.writeText(this.infoTarget.textContent)
  }
}
```
!!!

and import it in the layout like we did with the page component. 

If we have done this correctly we can navigate to our home page and see three buttons.   The first button puts our greeting in the message area when clicked, just like it did before, the second and third button fill the message area with "First clipboard item" and "Second clipboard item" respectively, it also copies the message to our clipboard. 

By using events we have allowed our page and copy components to communicate in a very controlled way.  The copy component is only allowed to trigger the methods we have exposed in the page controller, everything else stays hidden. 

### Parent to child communication

It is easy for a child component to trigger a parent action through events, for a parent element to trigger a child element with events we must attach the event to the window. Lets add the ability for the page to inform the copy component the event was received and we have changed the message.

in your page controller change your showMessage method and add a confirmEvent method like so: 

!!! code_block title="static/js/page.js"
```javascript
...

showMessage(message) {
  this.messageTarget.textContent = message
  this.confirmEvent()
}

confirmEvent() {
  this.dispatch("messageUpdated", {detail: {info: "Hello from parent"}})
}
```
!!!

in your copy component add `data-action="page:messageUpdated@window->copy#messageUpdated"` to the outer most div.

and add a method in our copy controller called messageUpdated like so: 

!!! code_block title="static/js/copy.js"
```javascript
...

messageUpdated(event) {
  this.element.style.backgroundColor = "green"
}
```
!!!

now run this code and you will see that when you click on the first or second copy button both have there background changed to green.  This is not what we wanted.  The desired behavior would be for only the first or second component to have the background color change.  This is of course because we had to attach the event to the window.  All the components are listening, so all change when any one event is dispatched.  To get around this issue we apply an id to the component and dispatch the event with that id in the detail.

we then check that id when we receive the event in the copy controller to ensure we should change the background color. 

in page.js change these three methods

!!! code_block title="static/js/page.js"
```javascript
showMessageEvent(event) {
  this.showMessage(event.detail.info, event.detail.id)
}

showMessage(message, id) {
  this.messageTarget.textContent = message
  this.confirmEvent(id)
}

confirmEvent(id) {
  this.dispatch("messageUpdated", {detail: {info: "Hello from parent", id: id}})
}
```
!!!

in copy.js change these methods

!!! code_block title="static/js/copy.js"
```javascript
showInfo() {
  this.dispatch("showMessage", {detail: {info: this.infoTarget.textContent, id: this.element.id}})
}

messageUpdated(event) {
  event.detail.id == this.element.id ? this.element.style.backgroundColor = "green" : null
}
```
!!!

add an id to the copy struct and pass that into the outer most div like so: 

!!! code_block components/copy/mod.rs
```rust
use sailfish::TemplateOnce;

#[derive(TemplateOnce, Default, Clone)]
#[template(path = "copy/template.html")]
pub struct Copy {
  ...
  id: String,
}

impl Copy {
    ...

    pub fn id(mut self, id: &str) -> Self {
        self.id = id.to_string();
        self
    }
}
```
!!!

and that should do it.  Now only the clicked component will change colors. 


Other methods exist for communicating between components such as directly invoking controllers and passing in actions and targets.  Directly invoking controllers exposes everything withing the controller from targets to methods, a powerful but unruly situation, as controller state can be altered by many actors.  Passing in actions and targets can be helpful for limiting a components exposed surface, but comes at the cost of being more confusing.  Since those are not recommended we will keep it for another post.  for now we should be able to do anything we need with this technique. 

## CSS

Adding css is very similar to how we added static js files.  I will leave it as an exercise to the reader to implement css styling. 

## Conclusion
It is important to remember there is no perfect or decided on solution, and we are still figuring it out.  It has been a learning process for us and the motivated reader will quickly find many tried and failed techniques in our code base.  When things are working we generally choose not to go back and change them with new techniques. We are optimizing speed and reliability, and refactoring can be a time consuming and risky business.  

Please reach out to us if you have any questions or suggestions.  Also, we encourage anyone interesting in developing in rust to look thorough our open source code base and contribute. 