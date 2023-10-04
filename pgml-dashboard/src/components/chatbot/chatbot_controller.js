import { Controller } from "@hotwired/stimulus";
import { createToast, showToast } from "../../../static/js/utilities/toast.js";

const LOADING_MESSAGE = `
<div class="d-flex align-items-end">
  <div>Loading</div>
  <div class="lds-ellipsis mb-2"><div></div><div></div><div></div><div></div></div>
</div>
`;

const getBackgroundImageURLForSide = (side, knowledgeBase) => {
  if (side == "user") {
    return "/dashboard/static/images/chatbot_user.webp";
  } else {
    if (knowledgeBase == 0) {
      return "/dashboard/static/images/owl_gradient.svg";
    } else if (knowledgeBase == 1) {
      return "/dashboard/static/images/logos/pytorch.svg";
    } else if (knowledgeBase == 2) {
      return "/dashboard/static/images/logos/rust.svg";
    } else if (knowledgeBase == 3) {
      return "/dashboard/static/images/logos/postgresql.svg";
    }
  }
};

const createHistoryMessage = (side, question, id, knowledgeBase) => {
  id = id || "";
  return `
  <div id="${id}" class="chatbot-message-wrapper pt-3 pb-3 ${
    side == "user" ? "chatbot-user-message" : "chatbot-bot-message"
  }">
      <div class="d-flex gap-1">
        <div>
          <div class="rounded p-1 chatbot-message-avatar-wrapper">
            <div class="chatbot-message-avatar" style="background-image: url('${getBackgroundImageURLForSide(
              side,
              knowledgeBase,
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

const knowledgeBaseIdToName = (knowledgeBase) => {
  if (knowledgeBase == 0) {
    return "PostgresML";
  } else if (knowledgeBase == 1) {
    return "PyTorch";
  } else if (knowledgeBase == 2) {
    return "Rust";
  } else if (knowledgeBase == 3) {
    return "PostgreSQL";
  }
};

const createKnowledgeBaseNotice = (knowledgeBase) => {
  return `
    <div class="chatbot-knowledge-base-notice text-center p-1">Chatting with Knowledge Base ${knowledgeBaseIdToName(
      knowledgeBase,
    )}</div>
  `;
};

const getAnswer = async (question, model, knowledgeBase) => {
  const response = await fetch("/chatbot/get-answer", {
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
      createHistoryMessage(
        "bot",
        LOADING_MESSAGE,
        "chatbot-loading-message",
        this.knowledgeBase,
      ),
    );
    this.hideExampleQuestions();
    this.chatHistory.scrollTop = this.chatHistory.scrollHeight;

    this.gettingAnswer = true;
    getAnswer(question, this.brain, this.knowledgeBase)
      .then((answer) => {
        if (answer.answer) {
          this.chatHistory.insertAdjacentHTML(
            "beforeend",
            createHistoryMessage(
              "bot",
              DOMPurify.sanitize(marked.parse(answer.answer)),
              "",
              this.knowledgeBase,
            ),
          );
        } else {
          this.showChatbotAlert("Error", answer.error);
          console.log(answer.error);
        }
      })
      .catch((error) => {
        this.showChatbotAlert("Error", "Error getting chatbot answer");
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
    // Comment this out when we go back to using brains
    this.brain = 0;
    this.questionInput.focus();

    // Uncomment this out when we go back to using brains
    // We could just disable the input, but we would then need to listen for click events so this seems easier
    // if (this.gettingAnswer) {
    //   document.querySelector(
    //     `input[name="chatbot-brain-options"][value="${this.brain}"]`,
    //   ).checked = true;
    //   this.showChatbotAlert(
    //     "Error",
    //     "Cannot change brain while chatbot is loading answer",
    //   );
    //   return;
    // }
    // let selected = parseInt(
    //   document.querySelector('input[name="chatbot-brain-options"]:checked')
    //     .value,
    // );
    // if (selected == this.brain) {
    //   return;
    // }
    // brainToContentMap[this.brain] = this.chatHistory.innerHTML;
    // this.chatHistory.innerHTML = brainToContentMap[selected] || "";
    // if (this.chatHistory.innerHTML) {
    //   this.exampleQuestions.style.setProperty("display", "none", "important");
    // } else {
    //   this.exampleQuestions.style.setProperty("display", "flex", "important");
    // }
    // this.brain = selected;
    // this.chatHistory.scrollTop = this.chatHistory.scrollHeight;
    // this.questionInput.focus();
  }

  handleKnowledgeBaseChange() {
    // Uncomment this when we go back to using brains
    // let selected = parseInt(
    //   document.querySelector('input[name="chatbot-knowledge-base-options"]:checked')
    //     .value,
    // );
    // this.knowledgeBase = selected;

    // Comment this out when we go back to using brains
    // We could just disable the input, but we would then need to listen for click events so this seems easier
    if (this.gettingAnswer) {
      document.querySelector(
        `input[name="chatbot-knowledge-base-options"][value="${this.knowledgeBase}"]`,
      ).checked = true;
      this.showChatbotAlert(
        "Error",
        "Cannot change knowledge base while chatbot is loading answer",
      );
      return;
    }
    let selected = parseInt(
      document.querySelector(
        'input[name="chatbot-knowledge-base-options"]:checked',
      ).value,
    );
    if (selected == this.knowledgeBase) {
      return;
    }

    // document.getElementById
    this.knowledgeBaseToContentMap[this.knowledgeBase] =
      this.chatHistory.innerHTML;
    this.chatHistory.innerHTML = this.knowledgeBaseToContentMap[selected] || "";
    this.knowledgeBase = selected;

    // This should be extended to insert the new knowledge base notice in the correct place
    if (this.chatHistory.childElementCount == 0) {
      this.chatHistory.insertAdjacentHTML(
        "beforeend",
        createKnowledgeBaseNotice(this.knowledgeBase),
      );
      this.hideExampleQuestions();
      document
        .getElementById(
          `chatbot-example-questions-${knowledgeBaseIdToName(
            this.knowledgeBase,
          )}`,
        )
        .style.setProperty("display", "flex", "important");
    } else if (this.chatHistory.childElementCount == 1) {
      this.hideExampleQuestions();
      document
        .getElementById(
          `chatbot-example-questions-${knowledgeBaseIdToName(
            this.knowledgeBase,
          )}`,
        )
        .style.setProperty("display", "flex", "important");
    } else {
      this.hideExampleQuestions();
    }

    this.chatHistory.scrollTop = this.chatHistory.scrollHeight;
    this.questionInput.focus();
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

// From: https://github.com/jackmoore/autosize, wrapped in an anonymous function
const autosize = (function () {
  let e = new Map();
  function t(t) {
    let o = e.get(t);
    o && o.destroy();
  }
  function o(t) {
    let o = e.get(t);
    o && o.update();
  }
  let r = null;
  return (
    "undefined" == typeof window
      ? (((r = (e) => e).destroy = (e) => e), (r.update = (e) => e))
      : (((r = (t, o) => (
          t &&
            Array.prototype.forEach.call(t.length ? t : [t], (t) =>
              (function t(o) {
                if (!o || !o.nodeName || "TEXTAREA" !== o.nodeName || e.has(o))
                  return;
                let r = null,
                  l = window.getComputedStyle(o);
                function n({
                  restoreTextAlign: e = null,
                  testForHeightReduction: t = !0,
                }) {
                  let i = l.overflowY;
                  if (0 === o.scrollHeight) return;
                  "vertical" === l.resize
                    ? (o.style.resize = "none")
                    : "both" === l.resize && (o.style.resize = "horizontal");
                  let s;
                  t &&
                    ((s = (function e(t) {
                      let o = [];
                      for (
                        ;
                        t && t.parentNode && t.parentNode instanceof Element;

                      )
                        t.parentNode.scrollTop &&
                          o.push([t.parentNode, t.parentNode.scrollTop]),
                          (t = t.parentNode);
                      return () =>
                        o.forEach(([e, t]) => {
                          (e.style.scrollBehavior = "auto"),
                            (e.scrollTop = t),
                            (e.style.scrollBehavior = null);
                        });
                    })(o)),
                    (o.style.height = ""));
                  let d;
                  if (
                    ((d =
                      "content-box" === l.boxSizing
                        ? o.scrollHeight -
                          (parseFloat(l.paddingTop) +
                            parseFloat(l.paddingBottom))
                        : o.scrollHeight +
                          parseFloat(l.borderTopWidth) +
                          parseFloat(l.borderBottomWidth)),
                    "none" !== l.maxHeight && d > parseFloat(l.maxHeight)
                      ? ("hidden" === l.overflowY &&
                          (o.style.overflow = "scroll"),
                        (d = parseFloat(l.maxHeight)))
                      : "hidden" !== l.overflowY &&
                        (o.style.overflow = "hidden"),
                    (o.style.height = d + "px"),
                    e && (o.style.textAlign = e),
                    s && s(),
                    r !== d &&
                      (o.dispatchEvent(
                        new Event("autosize:resized", { bubbles: !0 }),
                      ),
                      (r = d)),
                    i !== l.overflow && !e)
                  ) {
                    let a = l.textAlign;
                    "hidden" === l.overflow &&
                      (o.style.textAlign = "start" === a ? "end" : "start"),
                      n({ restoreTextAlign: a, testForHeightReduction: !0 });
                  }
                }
                function i() {
                  n({ testForHeightReduction: !0, restoreTextAlign: null });
                }
                let s,
                  d =
                    ((s = o.value),
                    () => {
                      n({
                        testForHeightReduction:
                          "" === s || !o.value.startsWith(s),
                        restoreTextAlign: null,
                      }),
                        (s = o.value);
                    }),
                  a = ((t) => {
                    o.removeEventListener("autosize:destroy", a),
                      o.removeEventListener("autosize:update", i),
                      o.removeEventListener("input", d),
                      window.removeEventListener("resize", i),
                      Object.keys(t).forEach((e) => (o.style[e] = t[e])),
                      e.delete(o);
                  }).bind(o, {
                    height: o.style.height,
                    resize: o.style.resize,
                    textAlign: o.style.textAlign,
                    overflowY: o.style.overflowY,
                    overflowX: o.style.overflowX,
                    wordWrap: o.style.wordWrap,
                  });
                o.addEventListener("autosize:destroy", a),
                  o.addEventListener("autosize:update", i),
                  o.addEventListener("input", d),
                  window.addEventListener("resize", i),
                  (o.style.overflowX = "hidden"),
                  (o.style.wordWrap = "break-word"),
                  e.set(o, { destroy: a, update: i }),
                  i();
              })(t, o),
            ),
          t
        )).destroy = (e) => (
          e && Array.prototype.forEach.call(e.length ? e : [e], t), e
        )),
        (r.update = (e) => (
          e && Array.prototype.forEach.call(e.length ? e : [e], o), e
        ))),
    r
  );
})();
