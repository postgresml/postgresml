import { Controller } from '@hotwired/stimulus'

export default class extends Controller {
  static targets = [
    "carousel", "carouselTimer"
  ]
  static outlets = []

  initialize() {
    this.times = 1;
    this.templates = document.getElementsByTagName("template");
    
  }

  connect() {
    this.test()
  }

  changeFeatured(next) {
    let current = this.carouselTarget.children[0]
    let nextItem = next.content.cloneNode(true)
    
    this.carouselTarget.appendChild(nextItem)

    if( current ) {
      current.style.marginLeft = "-100%";
      setTimeout( () => {
        this.carouselTarget.removeChild(current)
        console.log("timeout end: ", current)
      }, 700)
    }
  }

  changeIndicator(current, next) {
    console.log("in indicator")
    let timers = this.carouselTimerTargets;
    let currentTimer = timers[current];
    let nextTimer = timers[next]

    if ( currentTimer ) {
      console.log("current timer exists")
      currentTimer.classList.remove("timer-active")
      currentTimer.style.width = "1rem"
    }
    if( nextTimer) {
      console.log("in next timer") 
      nextTimer.style.width = "4rem"
      nextTimer.classList.add("timer-active")
   }
    console.log("indicator changed")
  }

  disconnect() {}

  testPause() {
    this.paused = true
  }

  testResume() {
    this.paused = false
  }

  test() {
    this.paused = false
    this.runtime = 0

    setInterval(() => {
      // maintain paused state through entire loop
      let paused = this.paused

      let activeTimer = document.getElementsByClassName("timer-active")[0]
      if( paused ) {
        if( activeTimer ) {
          activeTimer.classList.add("timer-pause")
        }
        console.log("paused")
      } else {
        if( activeTimer && activeTimer.classList.contains("timer-pause")) {
          activeTimer.classList.remove("timer-pause")
        }
        console.log("starting interval")
      }


      if( !paused && this.runtime % 5 == 0 ) {
        console.log("in action: ", paused, " ", this.runtime)
        let currentIndex = this.times % this.templates.length
        let nextIndex = (this.times + 1) % this.templates.length
    
        this.changeIndicator(currentIndex, nextIndex)
        this.changeFeatured(
          this.templates[nextIndex]
        )
        this.times ++
      }

      if( !paused ) {
        this.runtime++
      }
    }, 1000)
  }
}