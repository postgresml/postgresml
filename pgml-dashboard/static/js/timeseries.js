import {
    Controller
} from '@hotwired/stimulus';

export default class extends Controller {

    static values = {
        metricData: Object
    }

    connect() {
        // Plot on load and refresh button
        this.plot();

        // resize on navigation to metric tab
        const tabElement = document.querySelector('button[data-bs-target="#tab-Metrics"]');
        tabElement.addEventListener('shown.bs.tab', event => {
            this.plot()
        }, {once: true})
    }

    plot() {            
        const min = Math.min(...this.metricDataValue.values);
        const max = Math.max(...this.metricDataValue.values);
        const range = max-min;
        const color = "#ABACB0";
        const activeColor = "#F8FAFC";
        const lineColor = "#9185FF";
        const bgColor = "transparent"

        const trace = {
            x: this.metricDataValue.utc, 
            y: this.metricDataValue.values,
            fill: 'tonexty',
            mode: 'lines',
            line: {
              color: lineColor,
            },
        }

        const layout = {
            showlegend: false,
            plot_bgcolor: bgColor,
            paper_bgcolor: bgColor,
            height: document.body.offsetHeight*0.3,
            font: {
                color: color
            },
            margin: {b: 0, l: 0, r: 0, t: 40},
            yaxis: {
                range: [min-0.1*range, max+0.1*range],
                showgrid: false,
                automargin: true
            }, 
            xaxis: {
                showgrid: false,
                automargin: true
            },
            modebar: {
                activecolor: activeColor,
                bgcolor: bgColor,
                color: color,
                remove: ['autoscale', 'zoomin', 'zoomout']
            }
        }

        const config = {
            responsive: true,
            displaylogo: false
        }

        Plotly.newPlot(this.element.id, [trace], layout, config);
    }
}
