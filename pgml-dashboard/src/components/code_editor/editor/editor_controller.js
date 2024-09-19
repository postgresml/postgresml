import { Controller } from "@hotwired/stimulus";
import {
  generateModels,
  generateSql,
  generateOutput,
} from "../../../../static/js/utilities/demo";

export default class extends Controller {
  static targets = [
    "editor",
    "button",
    "loading",
    "result",
    "task",
    "model",
    "resultStream",
    "questionInput",
  ];

  static values = {
    defaultModel: String,
    defaultTask: String,
    runOnVisible: Boolean,
  };

  // Using an outlet is okay here since we need the exact instance of codeMirror
  static outlets = ["code-block"];

  // outlet callback not working so we listen for the
  // code-block to finish setting up CodeMirror editor view.
  codeBlockAvailable() {
    this.editor = this.codeBlockOutlet.getEditor();

    if (this.currentTask() !== "custom") {
      this.taskChange();
    }
    this.streaming = false;
    this.openConnection();
  }

  openConnection() {
    let protocol;
    switch (window.location.protocol) {
      case "http:":
        protocol = "ws";
        break;
      case "https:":
        protocol = "wss";
        break;
      default:
        protocol = "ws";
    }
    const url = `${protocol}://${window.location.host}/code_editor/play/stream`;

    this.socket = new WebSocket(url);

    if (this.runOnVisibleValue) {
      this.socket.addEventListener("open", () => {
        this.observe();
      });
    }

    this.socket.onmessage = (message) => {
      let result = JSON.parse(message.data);
      // We could probably clean this up
      if (result.error) {
        if (this.streaming) {
          this.resultStreamTarget.classList.remove("d-none");
          this.resultStreamTarget.innerHTML += result.error;
        } else {
          this.resultTarget.classList.remove("d-none");
          this.resultTarget.innerHTML += result.error;
        }
      } else {
        if (this.streaming) {
          this.resultStreamTarget.classList.remove("d-none");
          if (result.result == "\n") {
            this.resultStreamTarget.innerHTML += "</br></br>";
          } else {
            this.resultStreamTarget.innerHTML += result.result;
          }
          this.resultStreamTarget.scrollTop =
            this.resultStreamTarget.scrollHeight;
        } else {
          this.resultTarget.classList.remove("d-none");
          this.resultTarget.innerHTML += result.result;
        }
      }
      this.loadingTarget.classList.add("d-none");
      this.buttonTarget.disabled = false;
    };

    this.socket.onclose = () => {
      window.setTimeout(() => this.openConnection(), 500);
    };
  }

  onQuestionChange() {
    let transaction = this.editor.state.update({
      changes: {
        from: 0,
        to: this.editor.state.doc.length,
        insert: generateSql(
          this.currentTask(),
          this.currentModel(),
          this.questionInputTarget.value,
        ),
      },
    });
    this.editor.dispatch(transaction);
  }

  currentTask() {
    return this.hasTaskTarget ? this.taskTarget.value : this.defaultTaskValue;
  }

  currentModel() {
    return this.hasModelTarget
      ? this.modelTarget.value
      : this.defaultModelValue;
  }

  taskChange() {
    let models = generateModels(this.currentTask());
    let elements = this.element.querySelectorAll(".hh-m .menu-item");
    let allowedElements = [];

    for (let i = 0; i < elements.length; i++) {
      let element = elements[i];
      if (models.includes(element.getAttribute("data-for"))) {
        element.classList.remove("d-none");
        allowedElements.push(element);
      } else {
        element.classList.add("d-none");
      }
    }

    // Trigger a model change if the current one we have is not valid
    if (!models.includes(this.currentModel())) {
      allowedElements[0].firstElementChild.click();
    } else {
      let transaction = this.editor.state.update({
        changes: {
          from: 0,
          to: this.editor.state.doc.length,
          insert: generateSql(this.currentTask(), this.currentModel()),
        },
      });
      this.editor.dispatch(transaction);
    }
  }

  modelChange() {
    this.taskChange();
  }

  onSubmit(event) {
    event.preventDefault();
    this.buttonTarget.disabled = true;
    this.loadingTarget.classList.remove("d-none");
    this.resultTarget.classList.add("d-none");
    this.resultStreamTarget.classList.add("d-none");
    this.resultTarget.innerHTML = "";
    this.resultStreamTarget.innerHTML = "";

    // Update code area to include the users question.
    if (this.currentTask() == "embedded-query") {
      let transaction = this.editor.state.update({
        changes: {
          from: 0,
          to: this.editor.state.doc.length,
          insert: generateSql(
            this.currentTask(),
            this.currentModel(),
            this.questionInputTarget.value,
          ),
        },
      });
      this.editor.dispatch(transaction);
    }

    // Since db is read only, we show example result rather than sending request.
    if (this.currentTask() == "create-table") {
      this.resultTarget.innerHTML = generateOutput(this.currentTask());
      this.resultTarget.classList.remove("d-none");
      this.loadingTarget.classList.add("d-none");
      this.buttonTarget.disabled = false;
    } else {
      this.sendRequest();
    }
  }

  sendRequest() {
    let socketData = {
      sql: this.editor.state.doc.toString(),
    };

    if (this.currentTask() == "text-generation") {
      socketData.stream = true;
      this.streaming = true;
    } else {
      this.streaming = false;
    }

    this.lastSocketData = socketData;
    try {
      this.socket.send(JSON.stringify(socketData));
    } catch (e) {
      this.openConnection();
      this.socket.send(JSON.stringify(socketData));
    }
  }

  observe() {
    var options = {
      root: document.querySelector("#scrollArea"),
      rootMargin: "0px",
      threshold: 1.0,
    };

    let callback = (entries) => {
      entries.forEach((entry) => {
        if (entry.isIntersecting) {
          this.buttonTarget.click();
          this.observer.unobserve(this.element);
        }
      });
    };

    this.observer = new IntersectionObserver(callback, options);

    this.observer.observe(this.element);
  }
}
