function plot(timeseries_map) {
    document.getElementById('graph-main').innerHTML = '';
    const to_chart = []
    for (let k in timeseries_map) {
        const holder_id = `graph-holder-${k.replace(/ /g, "-")}`
        const canvas_id = `graph-${k.replace(/ /g, "-")}`;
        document.getElementById('graph-main').innerHTML += `
        <div id="${holder_id}" class="canvas_holder">
            <div id="graph-header-${k.replace(/ /g, "-")}" class="graph_header">
                <div id="graph-button-${k.replace(/ /g, "-")}" class="open open-close-button"></div>
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

        for (let point of timeseries_map[k]) {
            const ts = point.ts;
            const val = point.val;
            chart_obj.data.labels.push(ts);
            chart_obj.data.datasets[0].data.push(val);
            chart_obj.data.datasets[0].pointBackgroundColor.push('rgba(33,33,33 ,0.45)');
            chart_obj.data.datasets[0].pointBorderColor.push('rgba(33,33,33 ,0.45)');
        }
        to_chart.push([canvas_id, chart_obj]);
    }
    for(let chart of to_chart) {
        const ctx = document.getElementById(chart[0]).getContext('2d');
        const my_chart = new Chart(ctx, chart[1]);
    }
}


function default_timeseries_map(data, depth=0) {
    const timeseries_map = [];
    const new_rows = []
    data.forEach((row) => {
        try {
            const message_obj = JSON.parse(row[1]);
            const sender_name = row[0];
            const ts = row[2];

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
                    const timeseries_key = `${sender_name} - ${mkey}`;
                    if (timeseries_map[timeseries_key] === undefined) {
                        timeseries_map[timeseries_key] = [];
                    }
                    timeseries_map[timeseries_key].push({'ts': ts, 'val': parseInt(val)});
                }
            }
        } catch(err) {

        }
    });
    if(depth < 3) {
        depth++;
        const nm = default_timeseries_map(new_rows, depth);
        for (let k in nm) {
            timeseries_map[k] = nm[k];
        }
    }
    return timeseries_map;
}
