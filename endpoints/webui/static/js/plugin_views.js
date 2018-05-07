function plot(timeseries_map) {
    document.getElementById('graph-main').innerHTML = '';
    const to_chart = []
    for(let sender_name in timeseries_map) {
        const sender_holder_name = `${sender_name}-sender`;
        document.getElementById('graph-main').innerHTML += `
        <div id="graph-holder-${sender_holder_name}" class="sender-graph-holder">
            <div id="graph-header-${sender_holder_name}" class="sender-graph-header">
                <div id="graph-button-${sender_holder_name}" class="open-close-button"></div>
                ${sender_name}
            </div>
            <div id="graph-${sender_holder_name}" class="sender-graph" style="display: none;">
            </div>
        </div>`
        for (let k in timeseries_map[sender_name]) {
            const holder_id = `graph-holder-${sender_name}-${k.replace(/ /g, "-")}`
            const canvas_id = `graph-${sender_name}-${k.replace(/ /g, "-")}`;
            document.getElementById(`graph-${sender_holder_name}`).innerHTML += `
            <div id="${holder_id}" class="canvas-holder">
                <div id="graph-header-${sender_name}-${k.replace(/ /g, "-")}" class="graph-header">
                    <div id="graph-button-${sender_name}-${k.replace(/ /g, "-")}" class="open open-close-button"></div>
                </div>
                <canvas id="${canvas_id}"></canvas>
            </div>
            `;

            const chart_obj = {
                type: 'line',
                data: {
                    labels: [],
                    datasets: [{
                        label: k,
                        data: [],
                        pointBorderColor: [],
                        pointBackgroundColor: [],
                        backgroundColor: 'rgba(183,28,28,0)',
                        borderWidth: 2,
                        borderColor: 'rgba(183,28,28,1)',
                        pointRadius: 1
                    }]
                },
                options: {}
            }

            for (let point of timeseries_map[sender_name][k]) {
                const ts = point.ts;
                const val = point.val;
                chart_obj.data.labels.push(ts);
                chart_obj.data.datasets[0].data.push(val);
                chart_obj.data.datasets[0].pointBackgroundColor.push('rgba(33,33,33 ,0.45)');
                chart_obj.data.datasets[0].pointBorderColor.push('rgba(33,33,33 ,0.45)');
            }
            to_chart.push([canvas_id, chart_obj]);
        }
    }

    for(let chart of to_chart) {
        const ctx = document.getElementById(chart[0]).getContext('2d');
        const my_chart = new Chart(ctx, chart[1]);
    }
}


function default_timeseries_map(data, depth=0) {
    const timeseries_map = [];
    const new_rows = []
    const senders = []
    data.forEach((row) => {
        try {
            const message_obj = JSON.parse(row[1]);
            const sender_name = row[0];
            const ts = row[2];

            if ( !(senders.includes(sender_name)) ) {
                senders.push(sender_name);
            }
            for (let key in message_obj) {

                const val = message_obj[key];

                if (isNaN(val)) {
                    if(typeof val != "string" && typeof val != "boolean") {
                        let nval = val
                        if (val.constructor !== Array) {
                            nval = [val];
                        }
                        new_rows.push([sender_name, JSON.stringify(val) ,ts, key]);
                    }
                } else {
                    const mkey = row[3] === undefined ? key : `${row[3]} - ${key}`;
                    if (timeseries_map[sender_name] === undefined) {
                        timeseries_map[sender_name] = [];
                    }

                    if (timeseries_map[sender_name][mkey] === undefined) {
                        timeseries_map[sender_name][mkey] = [];
                    }
                    timeseries_map[sender_name][mkey].push({'ts': ts, 'val': parseFloat(val)});
                }
            }
        } catch(err) {

        }
    });
    if(depth < 3) {
        depth++;
        const nm = default_timeseries_map(new_rows, depth);
        for (let sender in nm) {
            for (let k in nm[sender]) {
                if(timeseries_map[sender] === undefined) {
                    timeseries_map[sender] = [];
                }
                timeseries_map[sender][k] = nm[sender][k];
            }
        }
    }
    return timeseries_map
}
