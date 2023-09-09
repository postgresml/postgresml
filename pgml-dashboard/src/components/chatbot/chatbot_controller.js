import { Controller } from "@hotwired/stimulus";

const createHistoryMessage = (side, question) => {
  return `
  <div class="chatbot-message-wrapper p-3 ${side == "user" ? 'chatbot-user-message' : 'chatbot-bot-message'}">
      <div class="d-flex gap-1">
        <div class="chatbot-message-avatar rounded p-2">
          <img
            class=""
            src=${side == "user" ? "/dashboard/static/images/chatbot_user.webp" : "/dashboard/static/images/owl_gradient.svg"}
            alt="Message Logo"
            width="44"
          />
        </div>
        <div class="chatbot-message pt-2">
          ${question}
        </div>
      </div>
    </div>
  `;
};

const brainToContentMap = {}

const getAnswer = async (question) => {
  const response = await fetch("/chatbot", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({ question }),
  });
  return response.json();
};

export default class extends Controller {
  initialize() {
    console.log("Initialized chatbot");
    this.questionInput = document.getElementById("chatbot-question-input");
    this.chatHistory = document.getElementById("chatbot-history");
    this.handleBrainChange(); // This will set our initial brain 
  }

  handleEnter(e) {
    // Prevents form from subbmiting
    // Turbo has a way to prevent in the html, but that is not working for me
    e.preventDefault();
    // Get the question and make sure it has some length greater than 0
    const question = this.questionInput.value.trim();
    if (question.length == 0) {
      return;
    }
    this.questionInput.value = "";
    this.chatHistory.insertAdjacentHTML(
      "beforeend",
      createHistoryMessage("user", question),
    );
    getAnswer(question)
      .then((answer) => {
        // Show answer
        this.chatHistory.insertAdjacentHTML(
          "beforeend",
          createHistoryMessage("bot", answer.answer),
        );
      })
      .catch((error) => {
        // Show error
      })
      .finally(() => {
        // Hide loading
      });

    console.log(question);
  }

  handleBrainChange() {
    let selected = document.querySelector('input[name="chatbot-brain-options"]:checked').value;
    if (selected == this.brain) {
      return;
    }
    brainToContentMap[this.brain] = this.chatHistory.innerHTML;
    this.chatHistory.innerHTML = brainToContentMap[selected] || "";
    this.brain = selected;
  }

  connect() { }

  disconnect() { }
}
