import { Controller } from "@hotwired/stimulus";
import { createToast, showToast } from "../../../static/js/utilities/toast.js";
import autosize from "autosize";
import DOMPurify from "dompurify";
import * as marked from "marked";

const getRandomInt = () => {
  return Math.floor(Math.random() * Number.MAX_SAFE_INTEGER);
}

const LOADING_MESSAGE = `
<div class="d-flex align-items-end">
  <div>Loading</div>
  <div class="lds-ellipsis mb-2"><div></div><div></div><div></div><div></div></div>
</div>
`;

const getBackgroundImageURLForSide = (side, brain) => {
  if (side == "user") {
    return "/dashboard/static/images/chatbot_user.webp";
  } else {
    if (brain == "teknium/OpenHermes-2.5-Mistral-7B") {
      return "/dashboard/static/images/logos/openhermes.webp"
    } else if (brain == "Gryphe/MythoMax-L2-13b") {
      return "/dashboard/static/images/logos/mythomax.webp"
    } else if (brain == "berkeley-nest/Starling-LM-7B-alpha") {
      return "/dashboard/static/images/logos/starling.webp"    
    } else if (brain == "openai") {
      return "/dashboard/static/images/logos/openai.webp"
    }
  }
};

const createHistoryMessage = (message) => {
  if (message.side == "system") {
    return `
      <div class="chatbot-knowledge-base-notice text-center p-3">${message.text}</div>
    `;
  }
  return `
  <div id="${message.id}" class="chatbot-message-wrapper pt-3 pb-3 ${
    message.side == "user" ? "chatbot-user-message" : "chatbot-bot-message"
  }">
      <div class="d-flex gap-1">
        <div>
          <div class="rounded p-1 chatbot-message-avatar-wrapper">
            <div class="chatbot-message-avatar" style="background-image: url('${getBackgroundImageURLForSide(
              message.side,
              message.brain,
            )}')">
          </div>
        </div>
        </div>
        <div class="chatbot-message ps-1 overflow-hidden" clean="true">
          ${message.get_html()}
        </div>
      </div>
    </div>
  `;
};

const knowledgeBaseIdToName = (knowledgeBase) => {
  if (knowledgeBase == "postgresml") {
    return "PostgresML";
  } else if (knowledgeBase == "pytorch") {
    return "PyTorch";
  } else if (knowledgeBase == "rust") {
    return "Rust";
  } else if (knowledgeBase == "postgresql") {
    return "PostgreSQL";
  }
};

const brainIdToName = (brain) => {
  if (brain == "teknium/OpenHermes-2.5-Mistral-7B") {
    return "OpenHermes"
  } else if (brain == "Gryphe/MythoMax-L2-13b") {
    return "MythoMax"
  } else if (brain == "berkeley-nest/Starling-LM-7B-alpha") {
    return "Starling"    
  } else if (brain == "openai") {
    return "ChatGPT"
  }
}

const createKnowledgeBaseNotice = (knowledgeBase) => {
  return `
    <div class="chatbot-knowledge-base-notice text-center p-3">Chatting with Knowledge Base ${knowledgeBaseIdToName(
      knowledgeBase,
    )}</div>
  `;
};

class Message {
  constructor(id, side, brain, text, is_partial=false) {
    this.id = id
    this.side = side
    this.brain = brain
    this.text = text
    this.is_partial = is_partial
  }

  get_html() {
    return DOMPurify.sanitize(marked.parse(this.text));
  }
}

class RawMessage extends Message {
  constructor(id, side, text, is_partial=false) {
    super(id, side, text, is_partial);
  }

  get_html() {
    return this.text;
  }
}

class MessageHistory {
  constructor() {
    this.messageHistory = {};
  }

  add_message(message, knowledgeBase) {
    console.log("ADDDING", message, knowledgeBase);
    if (!(knowledgeBase in this.messageHistory)) {
      this.messageHistory[knowledgeBase] = [];
    }
    if (message.is_partial) {
      let current_message = this.messageHistory[knowledgeBase].find(item => item.id == message.id);     
      if (!current_message) {
        this.messageHistory[knowledgeBase].push(message);
      } else {
        current_message.text += message.text;
      }
    } else {
      if (this.messageHistory[knowledgeBase].length == 0 || message.side != "system") {
          this.messageHistory[knowledgeBase].push(message);
      } else if (this.messageHistory[knowledgeBase][this.messageHistory[knowledgeBase].length -1].side == "system") {
        this.messageHistory[knowledgeBase][this.messageHistory[knowledgeBase].length -1] = message
      } else {
        this.messageHistory[knowledgeBase].push(message);
      }
    }
  }

  get_messages(knowledgeBase) {
    if (!(knowledgeBase in this.messageHistory)) {
      return [];
    } else {
      return this.messageHistory[knowledgeBase];
    }
  }
}

export default class extends Controller {
  initialize() {
    this.messageHistory = new MessageHistory();
    this.messageIdToKnowledgeBaseId = {};
    
    this.expanded = false;
    this.chatbot = document.getElementById("chatbot");
    this.expandContractImage = document.getElementById(
      "chatbot-expand-contract-image",
    );
    this.alertsWrapper = document.getElementById("chatbot-alerts-wrapper");
    this.questionInput = document.getElementById("chatbot-question-input");
    this.brainToContentMap = {};
    this.knowledgeBaseToContentMap = {};
    autosize(this.questionInput);
    this.chatHistory = document.getElementById("chatbot-history");
    this.exampleQuestions = document.getElementsByClassName(
      "chatbot-example-questions",
    );
    this.handleKnowledgeBaseChange(); // This will set our initial knowledge base
    this.handleBrainChange(); // This will set our initial brain
    this.handleResize();
    this.openConnection();
    this.getHistory();
  }

  openConnection() {
    const url = ((window.location.protocol === "https:") ? "wss://" : "ws://") + window.location.hostname + (((window.location.port != 80) && (window.location.port != 443)) ? ":" + window.location.port : "") + window.location.pathname + "/get-answer";
    this.socket = new WebSocket(url);
    this.socket.onmessage = (message) => {
      let result = JSON.parse(message.data);
      if (result.error) {
        this.showChatbotAlert("Error", "Error getting chatbot answer");
        console.log(result.error);
        this.redrawChat(); // This clears any loading messages
      } else {
        let message;
        if (result.partial_result) {
          message = new Message(result.id, "bot", this.brain, result.partial_result, true);
        } else {
          message = new Message(result.id, "bot", this.brain, result.result);
        }
        this.messageHistory.add_message(message, this.messageIdToKnowledgeBaseId[message.id]);
        this.redrawChat();
      }
      this.chatHistory.scrollTop = this.chatHistory.scrollHeight;
    };

    this.socket.onclose = () => {
      window.setTimeout(() => this.openConnection(), 500);
    };
  }

  async clearHistory() {
    // This endpoint clears the chatbot_sesion_id cookie
    await fetch("/chatbot/clear-history");
    window.location.reload();
  }

  async getHistory() {
    const result = await fetch("/chatbot/get-history");
    const history = await result.json();
    if (history.error) {
      console.log("Error getting chat history", history.error)
    } else {
      for (const message of history.result) {
        const newMessage = new Message(getRandomInt(), message.side, message.brain, message.content, false);
        console.log(newMessage);
        this.messageHistory.add_message(newMessage, message.knowledge_base);
      }
    }
    this.redrawChat();
  }

  redrawChat() {
    this.chatHistory.innerHTML = "";
    const messages = this.messageHistory.get_messages(this.knowledgeBase);
    for (const message of messages) {
      console.log("Drawing", message);
      this.chatHistory.insertAdjacentHTML(
        "beforeend",
        createHistoryMessage(message),
      );
    }

    // Hide or show example questions
    this.hideExampleQuestions();
    if (messages.length == 0 || (messages.length == 1 && messages[0].side == "system")) {
      document
        .getElementById(`chatbot-example-questions-${this.knowledgeBase}`)
        .style.setProperty("display", "flex", "important");
    }
    
    this.chatHistory.scrollTop = this.chatHistory.scrollHeight;
  }

  newUserQuestion(question) {
    const message = new Message(getRandomInt(), "user", this.brain, question);
    this.messageHistory.add_message(message, this.knowledgeBase);
    this.messageIdToKnowledgeBaseId[message.id] = this.knowledgeBase;
    this.hideExampleQuestions();
    this.redrawChat();

    let loadingMessage = new Message("loading", "bot", this.brain, LOADING_MESSAGE);
    this.chatHistory.insertAdjacentHTML(
      "beforeend",
      createHistoryMessage(loadingMessage),
    );
    this.chatHistory.scrollTop = this.chatHistory.scrollHeight;
    
    let id = getRandomInt();
    this.messageIdToKnowledgeBaseId[id] = this.knowledgeBase;
    let socketData = {
      id,
      question,
      model: this.brain,
      knowledge_base: this.knowledgeBase
    };
    this.socket.send(JSON.stringify(socketData));
  }

  handleResize() {
    if (this.expanded && window.innerWidth >= 1000) {
      this.chatbot.classList.add("chatbot-full");
    } else {
      this.chatbot.classList.remove("chatbot-full");
    }

    let html = this.chatHistory.innerHTML;
    this.chatHistory.innerHTML = "";
    let height = this.chatHistory.offsetHeight;
    this.chatHistory.style.height = height + "px";
    this.chatHistory.innerHTML = html;
    this.chatHistory.scrollTop = this.chatHistory.scrollHeight;
  }

  handleEnter(e) {
    // This prevents adding a return
    e.preventDefault();
    // Don't continue if the question is empty
    const question = this.questionInput.value.trim();
    if (question.length == 0)
      return;
    // Handle resetting the input
    // There is probably a better way to do this, but this was the best/easiest I found
    this.questionInput.value = "";
    autosize.destroy(this.questionInput);
    autosize(this.questionInput);

    this.newUserQuestion(question);
  }

  handleBrainChange() {
    let selected = document.querySelector('input[name="chatbot-brain-options"]:checked').value;
    if (selected == this.brain)
      return;
    this.brain = selected;
    this.questionInput.focus();
    this.addBrainAndKnowledgeBaseChangedSystemMessage();
  }

  handleKnowledgeBaseChange() {
    let selected = document.querySelector('input[name="chatbot-knowledge-base-options"]:checked').value;
    if (selected == this.knowledgeBase)
      return;
    this.knowledgeBase = selected;
    this.redrawChat();
    this.questionInput.focus();
    this.addBrainAndKnowledgeBaseChangedSystemMessage();
  }

  addBrainAndKnowledgeBaseChangedSystemMessage() {
    let knowledge_base = knowledgeBaseIdToName(this.knowledgeBase);
    let brain = brainIdToName(this.brain);
    let content = `Chatting with ${brain} about ${knowledge_base}`;
    const newMessage = new Message(getRandomInt(), "system", this.brain, content);
    this.messageHistory.add_message(newMessage, this.knowledgeBase);
    this.redrawChat();
  }

  handleExampleQuestionClick(e) {
    const question = e.currentTarget.getAttribute("data-value");
    this.newUserQuestion(question);
  }

  handleExpandClick() {
    this.expanded = !this.expanded;
    this.chatbot.classList.toggle("chatbot-expanded");
    if (this.expanded) {
      this.expandContractImage.src =
        "/dashboard/static/images/icons/arrow_compressed.svg";
    } else {
      this.expandContractImage.src =
        "/dashboard/static/images/icons/arrow_expanded.svg";
    }
    this.handleResize();
    this.questionInput.focus();
  }

  showChatbotAlert(level, message) {
    const toastElement = createToast(message, level);
    showToast(toastElement, {
      autohide: true,
      delay: 7000
    });
  }

  hideExampleQuestions() {
    for (let i = 0; i < this.exampleQuestions.length; i++) {
      this.exampleQuestions
        .item(i)
        .style.setProperty("display", "none", "important");
    }
  }
}
