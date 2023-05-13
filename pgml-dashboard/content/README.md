## Syntax Documentation for Content

PostgresML documentation is written in markdown and uses [Comrak](https://github.com/kivikakk/comrak) for parsing.  This provides the author with all standard markdown styling. In addition, we add richness to our documentation with custom widgets.

You can see all widgets rendered at [style guide](https://postgresml.org/blog/style_guide).

### Tabs 

Tabs are excellent for reducing clutter on a page and grouping information.  Use the following syntax to create a tab widget. 

````markdown
=== "Tab 1"

information in the first tab

=== "Tab 2"

information in the second tab

===
````

### Admonitions

Admonitions, or call-outs, are a great way to bring attention to important information.  

We us `!!!` to signal an admonition. The general syntax to create an admonition is 

``` 
!!! {name-of-admonition}

{your text}

!!!
```

For example the following code is how you create a note admonition. 
```
!!! Note 

This is a note admonition

!!!
```

The admonitions available are 
 - Note
 - Abstract
 - Info
 - Tip
 - Example
 - Question
 - Success
 - Quote
 - Bug
 - Warning
 - Fail
 - Danger 
 - Generic

### Code 

PostgresML has many different styles available for showing code.  

#### Inline Code 

Use standard markdown syntax for inline code. 

#### Fenced Code

Use standard markdown syntax for fenced code.  All fenced code will have a toolbar attached to the upper right hand corner.  It contains a copy feature, other features will be added in the future. 

#### Code Block

To make code standout more, the author can apply a title, execution time, and border to their code using our custom code_block widget.  The title and execution time are optional. The following syntax renders a code block with a title "Code" and an execution time "21ms". 

````markdown
!!! code_block title="Code Title" time="21ms"

``` sql
Your code goes here
```

!!!

````

#### Results 

The author can show code results using the results widget.  Results widgets will render code differently than code blocks.  This makes it clear to the reader if the code is output or input. Render a results block with the following syntax. 

```` markdown
!!! results title="Your Code Title"

``` your code results here ```

!!!
````

or 

```` markdown
!!! results title="Your Code Title"

your non code or table results here 

!!!
````

#### Suggestion

An excellent way to bring attention to your code is to use a generic admonition with a code block and a results block.  

```` markdown
!!! generic 

!!! code_block title="Your Code Title" time="21ms"

``` Some code to execute ```

!!!

!!! results title="Results Title"

``` Your Code Results ``` 

!!!

!!!
````
