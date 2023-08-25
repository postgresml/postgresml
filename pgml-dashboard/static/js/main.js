export function linearRegression(x, y) {
    var lr = {};
    var n = y.length;
    var sum_x = 0;
    var sum_y = 0;
    var sum_xy = 0;
    var sum_xx = 0;
    var sum_yy = 0;

    for (var i = 0; i < y.length; i++) {
        sum_x += x[i];
        sum_y += y[i];
        sum_xy += (x[i] * y[i]);
        sum_xx += (x[i] * x[i]);
        sum_yy += (y[i] * y[i]);
    }

    lr['sl'] = (n * sum_xy - sum_x * sum_y) / (n * sum_xx - sum_x * sum_x);
    lr['off'] = (sum_y - lr.sl * sum_x) / n;
    lr['r2'] = Math.pow((n * sum_xy - sum_x * sum_y) / Math.sqrt((n * sum_xx - sum_x * sum_x) * (n * sum_yy - sum_y * sum_y)), 2);

    return lr;
}

export function renderModel(model_id, key_metric, range) {
    Plotly.newPlot(
        "model_" + model_id,
        [{
            type: "bar",
            orientation: "h",
            x: [0],
            marker: {
                color: "rgb(0, 180, 255)",
            },
            hoverinfo: "none",
        }],
        {
            xaxis: {
                range: range,
                showgrid: false,
                zeroline: false,
                visible: false,
            },
            yaxis: {
                showgrid: false,
                zeroline: false,
                visible: false,
            },
            paper_bgcolor: 'rgba(0,0,0,0)',
            plot_bgcolor: 'rgba(0,0,0,0)',
            margin: { l: 0, r: 0, b: 0, t: 0, pad: 0 },
            scrollZoom: false,
            dragmode: false,
        },
        { displayModeBar: false, responsive: true }
    )
    Plotly.animate(
        "model_" + model_id,
        {
            data: [{ x: [key_metric] }],
            traces: [0],
            layout: {}
        }, {
        transition: {
            duration: 500,
            easing: 'cubic-in-out'
        },
        frame: {
            duration: 500
        }
    }
    )
}

export function renderDistribution(feature_name, samples, dip) {
    Plotly.newPlot(
        feature_name + '_distribution',
        [{
            type: "histogram",
            name: feature_name,
            x: samples,
            marker: {
                color: "rgba(0, 180, 255, 0.7)",
                line: {
                    color: "rgba(0, 180, 255, 1)",
                    width: 1
                },
            },
        }],
        {
            title: {
                font: { family: "Roboto, \"Microsoft Sans Serif\", \"Helvetica Neue\", Arial, sans-serif" },
                text: "Distribution: dip = " + (Math.round(dip * 100) / 100).toString(),
            },
            hoverlabel: {
                color: "rgba(0, 180, 255, 0.7)",
                bordercolor: "rgba(255, 255, 255, 1)",
                font: {
                    color: "rgba(255, 255, 255, 1)",
                    family: "Roboto, \"Microsoft Sans Serif\", \"Helvetica Neue\", Arial, sans-serif",
                    size: 12
                },
            },
            paper_bgcolor: 'rgba(0,0,0,0)',
            plot_bgcolor: 'rgba(0,0,0,0)',
            yaxis: { automargin: true },
            xaxis: { automargin: true },
            margin: { l: 0, r: 0, b: 0, t: 50, pad: 0 },
            scrollZoom: false,
            dragmode: false,
        },
        { displayModeBar: false, responsive: true }
    );
}

export function renderOutliers(feature_name, samples, stddev) {
    Plotly.newPlot(
        feature_name + '_outliers',
        [{
            type: "box",
            jitter: 0.3,
            pointpos: -1.8,
            boxpoints: 'all',
            name: feature_name,
            y: samples,
            marker: {
                color: "rgba(0, 180, 255, 0.3)",
                line: {
                    color: "rgba(0, 180, 255, 0.5)",
                    width: 1
                },
            },
        }],
        {
            title: {
                font: { family: "Roboto, \"Microsoft Sans Serif\", \"Helvetica Neue\", Arial, sans-serif" },
                text: "Outliers: σ = " + (Math.round(stddev * 100) / 100).toString(),
            },
            hoverlabel: {
                bgcolor: "rgba(0, 180, 255, 0.7)",
                bordercolor: "rgba(255, 255, 255, 1)",
                font: {
                    color: "rgba(255, 255, 255, 1)",
                    family: "Roboto, \"Microsoft Sans Serif\", \"Helvetica Neue\", Arial, sans-serif",
                    size: 12
                },
            },
            paper_bgcolor: 'rgba(0,0,0,0)',
            plot_bgcolor: 'rgba(0,0,0,0)',
            yaxis: { automargin: true },
            xaxis: { automargin: true },
            margin: { l: 0, r: 0, b: 0, t: 50, pad: 0 },
            scrollZoom: false,
            dragmode: false,
        },
        { displayModeBar: false, responsive: true }
    );
}

export function renderCorrelation(feature_name, other_name, samples, y) {
    var lr = linearRegression(samples, y);
    var fit_from = Math.min(...samples);
    var fit_to = Math.max(...samples);

    Plotly.newPlot(
        feature_name + '_correlation_' + other_name,
        [{
            type: "scatter",
            mode: "markers",
            name: feature_name,
            x: samples,
            y: y,
            marker: {
                color: "rgba(0, 180, 255, 0.4)",
                line: {
                    color: "rgba(0, 180, 255, 0.6)",
                    width: 1
                },
            },
        }, {
            type: 'scatter',
            mode: 'lines',
            name: "R<sup>2</sup> = " + (Math.round(lr.r2 * 100) / 100).toString(),
            x: [fit_from, fit_to],
            y: [fit_from * lr.sl + lr.off, fit_to * lr.sl + lr.off],
            marker: {
                color: "rgba(255, 232, 20, 1)",
                line: {
                    color: "rgba(255, 232, 20, 1)",
                    width: 2
                },
            },
        }],
        {
            font: { family: "Roboto, \"Microsoft Sans Serif\", \"Helvetica Neue\", Arial, sans-serif" },
            title: "Correlation: R<sup>2</sup> = " + (Math.round(lr.r2 * 100) / 100).toString(),
            hoverlabel: {
                bgcolor: "rgba(0, 180, 255, 0.7)",
                bordercolor: "rgba(255, 255, 255, 1)",
                font: {
                    color: "rgba(255, 255, 255, 1)",
                    family: "Roboto, \"Microsoft Sans Serif\", \"Helvetica Neue\", Arial, sans-serif",
                    size: 12
                },
            },
            paper_bgcolor: 'rgba(0,0,0,0)',
            plot_bgcolor: 'rgba(0,0,0,0)',
            yaxis: { automargin: true },
            xaxis: { automargin: true },
            margin: { l: 0, r: 0, b: 0, t: 50, pad: 0 },
            showlegend: false,
            scrollZoom: false,
            dragmode: false,

        },
        { displayModeBar: false, responsive: true }
    );
}

export function renderHyperparam(id, name, title, param, best_index, mean, std) {
    Plotly.newPlot(
        id,
        [{
            type: "scatter",
            mode: "markers",
            name: name,
            x: param,
            y: mean,
            error_y: {
                type: 'data',
                array: std,
                visible: true,
                color: "rgba(0, 180, 255, 0.05)"
            },
            marker: {
                color: "rgba(0, 180, 255, 0.4)",
                line: {
                    color: "rgba(0, 180, 255, 0.6)",
                    width: 1
                },
            },
        },
        {
            type: "scatter",
            mode: "markers",
            name: name,
            x: [param[best_index]],
            y: [mean[best_index]],
            error_y: {
                type: 'data',
                type: 'constant',
                value: std[best_index],
                visible: true,
                color: "rgba(255, 43, 158, 0.5)"
            },
            marker: {
                color: "rgba(255, 43, 158, 0.7)",
                line: {
                    color: "rgba(255, 43, 158, 1.0)",
                    width: 1
                },
            },
        }],
        {
          font: { family: "Roboto, \"Microsoft Sans Serif\", \"Helvetica Neue\", Arial, sans-serif" },
          title: title,
          hoverlabel: {
            bgcolor: "rgba(0, 180, 255, 0.7)",
            bordercolor: "rgba(255, 255, 255, 1)",
            font: {
              color: "rgba(255, 255, 255, 1)",
              family: "Roboto, \"Microsoft Sans Serif\", \"Helvetica Neue\", Arial, sans-serif",
              size: 12
            },
          },
          paper_bgcolor: 'rgba(0,0,0,0)',
          plot_bgcolor: 'rgba(0,0,0,0)',
          yaxis: { automargin: true },
          xaxis: { automargin: true },
          margin: { l: 0, r: 0, b: 0, t: 50, pad: 0 },
          showlegend: false,
          scrollZoom: false,
          dragmode: false,
        },
        { displayModeBar: false, responsive: true }
    );
}
