import {
    Controller
} from '@hotwired/stimulus'

export default class extends Controller {
    static targets = [
        'btnSecondary',
    ]

    connect() {
        this.respondToVisibility()
    }

    // Hook for when the secondary btn is in viewport
    respondToVisibility() {  
        let options = {
            root: null, 
            rootMargin: "0px"
        }

        var observer = new IntersectionObserver((entries) => {
            entries.forEach((entry) => {
                if (entry.isIntersecting) {
                    this.attachCanvas()
                }
            });
        }, options);

        observer.observe(this.btnSecondaryTarget);
    }
    
    attachCanvas() {
        let btn = this.btnSecondaryTarget
        let canvasElements = btn.getElementsByTagName("canvas")

        if (canvasElements.length) {
            var canvas = canvasElements[0]
        } else {
            var canvas = document.createElement("canvas")
            canvas.className = "secondary-btn-canvas"
        }
        
        btn.appendChild(canvas)
        this.drawBorder(btn, canvas)
    }

    drawBorder(btn, canvas) {
        let btnMarginX = 22
        let btnMarginY = 12
        let borderRadius = 8;
        let width = btn.offsetWidth
        let height = btn.offsetHeight

    
        canvas.width = width
        canvas.height = height
        canvas.style.margin = `-${height - btnMarginY}px -${btnMarginX}px`
        if( !width ) {
            return
        }
        
        // Draw border compensating for border thickenss
        var ctx = canvas.getContext("2d")
        ctx.moveTo(borderRadius, 1)
        ctx.lineTo(width-borderRadius-1, 1)
        ctx.arcTo(width-1, 1, width-1, borderRadius-1, borderRadius-1)
        ctx.arcTo(width-1, height-1, width-borderRadius-1, height-1, borderRadius-1)
        ctx.lineTo(borderRadius-1, height-1)
        ctx.arcTo(1, height-1, 1, borderRadius-1, borderRadius-1)
        ctx.arcTo(1, 1, borderRadius-1, 1, borderRadius-1)
    
        var gradient = ctx.createLinearGradient(0, canvas.height, canvas.width, 0)
        gradient.addColorStop(0, "rgb(217, 64, 255)");
        gradient.addColorStop(0.24242424242424243, "rgb(143, 2, 254)");
        gradient.addColorStop(0.5606060606060606, "rgb(81, 98, 255)");
        gradient.addColorStop(1, "rgb(0, 209, 255)");
      
        // Fill with gradient
        ctx.strokeStyle = gradient
        ctx.lineWidth = 2
        ctx.stroke()
    }
}
