import { Controller } from '@hotwired/stimulus'

export default class extends Controller {
  static targets = [
    "carousel", "carouselTimer", "template"
  ]

  initialize() {
    this.paused = false
    this.runtime = 0
    this.times = 1;
  }

  connect() {
    // dont cycle carousel if it only hase one item. 
    if ( this.templateTargets.length > 1 ) {
      this.cycle()
    }
  }

  changeFeatured(next) {
    let current = this.carouselTarget.children[0]
    let nextItem = next.content.cloneNode(true)
    
    this.carouselTarget.appendChild(nextItem)

    if( current ) {
      current.style.marginLeft = "-100%";
      setTimeout( () => {
        this.carouselTarget.removeChild(current)
      }, 700)
    }
  }

  changeIndicator(current, next) {
    let timers = this.carouselTimerTargets;
    let currentTimer = timers[current];
    let nextTimer = timers[next]

    if ( currentTimer ) {
      currentTimer.classList.remove("timer-active")
      currentTimer.style.width = "1rem"
    }
    if( nextTimer) {
      nextTimer.style.width = "4rem"
      nextTimer.classList.add("timer-active")
   }
  }

  Pause() {
    this.paused = true
  }

  Resume() {
    this.paused = false
  }

  cycle() {
    this.interval = setInterval(() => {
      // maintain paused state through entire loop
      let paused = this.paused

      let activeTimer = document.getElementsByClassName("timer-active")[0]
      if( paused ) {
        if( activeTimer ) {
          activeTimer.classList.add("timer-pause")
        }
      } else {
        if( activeTimer && activeTimer.classList.contains("timer-pause")) {
          activeTimer.classList.remove("timer-pause")
        }
      }

      if( !paused && this.runtime % 5 == 0 ) {
        let currentIndex = this.times % this.templateTargets.length
        let nextIndex = (this.times + 1) % this.templateTargets.length
    
        this.changeIndicator(currentIndex, nextIndex)
        this.changeFeatured(
          this.templateTargets[nextIndex]
        )
        this.times ++
      }

      if( !paused ) {
        this.runtime++
      }
    }, 1000)
  }

  disconnect() {
    clearInterval(this.interval);
  }
}
