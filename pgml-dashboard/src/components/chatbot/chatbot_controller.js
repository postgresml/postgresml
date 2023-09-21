import { Controller } from "@hotwired/stimulus";

const LOADING_MESSAGE = `
<div class="d-flex align-items-end">
  <div>Loading</div>
  <div class="lds-ellipsis mb-2"><div></div><div></div><div></div><div></div></div>
</div>
`;

const getBackgroundImageURLForSide = (side) => {
  return `${side == "user"
      ? "/dashboard/static/images/chatbot_user.webp"
      : "/dashboard/static/images/owl_gradient.svg"
    }`;
};

const createHistoryMessage = (side, question, id) => {
  id = id || "";
  return `
  <div id="${id}" class="chatbot-message-wrapper pt-3 pb-3 ${side == "user" ? "chatbot-user-message" : "chatbot-bot-message"
    }">
      <div class="d-flex gap-1">
        <div>
          <div class="rounded p-1 chatbot-message-avatar-wrapper">
            <div class="chatbot-message-avatar" style="background-image: url('${getBackgroundImageURLForSide(
      side,
    )}')">
          </div>
        </div>
        </div>
        <div class="chatbot-message ps-1">
          ${question}
        </div>
      </div>
    </div>
  `;
};

const brainToContentMap = {};

const getAnswer = async (question, model, knowledgeBase) => {
  const response = await fetch("/chatbot", {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify({ question, model, knowledgeBase }),
  });
  return response.json();
};

export default class extends Controller {
  initialize() {
    this.alertCount = 0;
    this.gettingAnswer = false;
    this.expanded = false;
    this.chatbot = document.getElementById("chatbot");
    this.alertsWrapper = document.getElementById("chatbot-alerts-wrapper");
    this.questionInput = document.getElementById("chatbot-question-input");
    autosize(this.questionInput);
    this.chatHistory = document.getElementById("chatbot-history");
    this.exampleQuestions = document.getElementById(
      "chatbot-example-questions",
    );
    this.handleBrainChange(); // This will set our initial brain
    this.handleKnowledgeBaseChange(); // This will set our initial knowledge base
    this.handleResize();
  }

  newUserQuestion(question) {
    this.chatHistory.insertAdjacentHTML(
      "beforeend",
      createHistoryMessage("user", question),
    );
    this.chatHistory.insertAdjacentHTML(
      "beforeend",
      createHistoryMessage("bot", LOADING_MESSAGE, "chatbot-loading-message"),
    );
    this.exampleQuestions.style.setProperty("display", "none", "important");
    console.log(this.exampleQuestions);
    console.log(this.exampleQuestions.style.display);
    console.log("SETTING EXAMPLE QUESTIONS TO HIDDEN");
    this.chatHistory.scrollTop = this.chatHistory.scrollHeight;

    this.gettingAnswer = true;
    getAnswer(question, this.brain, this.knowledgeBase)
      .then((answer) => {
        this.chatHistory.insertAdjacentHTML(
          "beforeend",
          createHistoryMessage("bot", answer.answer),
        );
      })
      .catch((error) => {
        console.log(error);
      })
      .finally(() => {
        document.getElementById("chatbot-loading-message").remove();
        this.chatHistory.scrollTop = this.chatHistory.scrollHeight;
        this.gettingAnswer = false;
      });
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

    const question = this.questionInput.value.trim();
    if (question.length == 0) {
      return;
    }

    // Handle resetting the input
    // There is probably a better way to do this, but this was the best/easiest I found
    this.questionInput.value = "";
    autosize.destroy(this.questionInput);
    autosize(this.questionInput);

    this.newUserQuestion(question);
  }

  handleBrainChange() {
    // We could just disable the input, but we would then need to listen for click events so this seems easier
    if (this.gettingAnswer) {
      document.querySelector(`input[name="chatbot-brain-options"][value="${this.brain}"]`).checked = true;
      this.showChatbotAlert("Error", "Cannot change brain while chatbot is loading answer");
      return;
    }
    let selected = parseInt(
      document.querySelector('input[name="chatbot-brain-options"]:checked')
        .value,
    );
    if (selected == this.brain) {
      return;
    }
    brainToContentMap[this.brain] = this.chatHistory.innerHTML;
    this.chatHistory.innerHTML = brainToContentMap[selected] || "";
    if (this.chatHistory.innerHTML) {
      this.exampleQuestions.style.setProperty("display", "none", "important");
    } else {
      this.exampleQuestions.style.setProperty("display", "flex", "important");
    }
    this.brain = selected;
    this.chatHistory.scrollTop = this.chatHistory.scrollHeight;
    this.questionInput.focus();
  }

  handleKnowledgeBaseChange() {
    let selected = parseInt(
      document.querySelector('input[name="chatbot-brain-options"]:checked')
        .value,
    );
    this.knowledgeBase = selected;
  }

  handleExampleQuestionClick(e) {
    const question = e.currentTarget.getAttribute("data-value");
    this.newUserQuestion(question);
  }

  handleExpandClick() {
    this.expanded = !this.expanded;
    this.chatbot.classList.toggle("chatbot-expanded");
    this.handleResize();
    this.questionInput.focus();
  }

  showChatbotAlert = (level, message) => {
    const alertCount = this.alertCount;
    this.alertCount += 1;
    let alertHTML = `
      <div id="chatbot-alert-${alertCount}" class="alert alert-primary alert-dismissible fade show" role="alert">
        <strong>${level}</strong> <span>${message}</span>
        <button type="button" class="btn-close" data-bs-dismiss="alert" aria-label="Close"></button>
      </div>
    `;
    this.alertsWrapper.insertAdjacentHTML("afterbegin", alertHTML);
    window.setTimeout(() => {
      document.getElementById(`chatbot-alert-${alertCount}`).remove();
    }, 5000);
  };
}

// From: https://github.com/jackmoore/autosize, wrapped in an anonymous function and then minified
const autosize = function() { let e = new Map; function t(t) { let o = e.get(t); o && o.destroy() } function o(t) { let o = e.get(t); o && o.update() } let r = null; return "undefined" == typeof window ? ((r = e => e).destroy = e => e, r.update = e => e) : ((r = (t, o) => (t && Array.prototype.forEach.call(t.length ? t : [t], t => (function t(o) { if (!o || !o.nodeName || "TEXTAREA" !== o.nodeName || e.has(o)) return; let r = null, l = window.getComputedStyle(o); function n({ restoreTextAlign: e = null, testForHeightReduction: t = !0 }) { let i = l.overflowY; if (0 === o.scrollHeight) return; "vertical" === l.resize ? o.style.resize = "none" : "both" === l.resize && (o.style.resize = "horizontal"); let s; t && (s = function e(t) { let o = []; for (; t && t.parentNode && t.parentNode instanceof Element;)t.parentNode.scrollTop && o.push([t.parentNode, t.parentNode.scrollTop]), t = t.parentNode; return () => o.forEach(([e, t]) => { e.style.scrollBehavior = "auto", e.scrollTop = t, e.style.scrollBehavior = null }) }(o), o.style.height = ""); let d; if (d = "content-box" === l.boxSizing ? o.scrollHeight - (parseFloat(l.paddingTop) + parseFloat(l.paddingBottom)) : o.scrollHeight + parseFloat(l.borderTopWidth) + parseFloat(l.borderBottomWidth), "none" !== l.maxHeight && d > parseFloat(l.maxHeight) ? ("hidden" === l.overflowY && (o.style.overflow = "scroll"), d = parseFloat(l.maxHeight)) : "hidden" !== l.overflowY && (o.style.overflow = "hidden"), o.style.height = d + "px", e && (o.style.textAlign = e), s && s(), r !== d && (o.dispatchEvent(new Event("autosize:resized", { bubbles: !0 })), r = d), i !== l.overflow && !e) { let a = l.textAlign; "hidden" === l.overflow && (o.style.textAlign = "start" === a ? "end" : "start"), n({ restoreTextAlign: a, testForHeightReduction: !0 }) } } function i() { n({ testForHeightReduction: !0, restoreTextAlign: null }) } let s, d = (s = o.value, () => { n({ testForHeightReduction: "" === s || !o.value.startsWith(s), restoreTextAlign: null }), s = o.value }), a = (t => { o.removeEventListener("autosize:destroy", a), o.removeEventListener("autosize:update", i), o.removeEventListener("input", d), window.removeEventListener("resize", i), Object.keys(t).forEach(e => o.style[e] = t[e]), e.delete(o) }).bind(o, { height: o.style.height, resize: o.style.resize, textAlign: o.style.textAlign, overflowY: o.style.overflowY, overflowX: o.style.overflowX, wordWrap: o.style.wordWrap }); o.addEventListener("autosize:destroy", a), o.addEventListener("autosize:update", i), o.addEventListener("input", d), window.addEventListener("resize", i), o.style.overflowX = "hidden", o.style.wordWrap = "break-word", e.set(o, { destroy: a, update: i }), i() })(t, o)), t)).destroy = e => (e && Array.prototype.forEach.call(e.length ? e : [e], t), e), r.update = e => (e && Array.prototype.forEach.call(e.length ? e : [e], o), e)), r }();
