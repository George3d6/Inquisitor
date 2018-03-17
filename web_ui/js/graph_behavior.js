function close_button_behavior() {
    const buttons = document.getElementsByClassName('open-close-button');
    for(let i = 0; i < buttons.length; i++) {
        buttons[i].addEventListener('click', () => {
            let is_open = false;
            let classes = '';
            buttons[i].className.split(' ').forEach((cls) => {
                if(cls === 'open') {
                    is_open = true;
                } else {
                    classes += `${cls} `;
                }
            });

            const graph_id = buttons[i].id.replace('button-', '');
            const graph = document.getElementById(graph_id);

            const graph_holder_id = buttons[i].id.replace('button-', 'holder-');
            const graph_holder = document.getElementById(graph_holder_id);


            if(!is_open) {
                classes += ` open`;
                graph.style.display = 'block';
                //graph_holder.style.height =
            } else {
                graph.style.display = 'none';
            }
            buttons[i].className = classes;
        });
    }
}
