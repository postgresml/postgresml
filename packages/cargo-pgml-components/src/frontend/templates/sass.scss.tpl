div[data-controller="<%= component.controller_name() %>"] {
    // Used to identify the component in the DOM.
    // Delete these styles if you don't need them.
    min-width: 100px;
    width: 100%;
    height: 100px;

    background: red;

    display: flex;
    justify-content: center;
    align-items: center;

    h3 {
        color: white;
    }
}
