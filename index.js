import {Optim, Uuid, MaintenanceSummary } from "maintenance_site";


const input_file = document.getElementById('file');
const send_file = document.getElementById('send-file');
const send_file_2 = document.getElementById('send-file-json');
const planning = document.getElementById('planning');
const gl_planning = planning.getContext('webgl');


const CELL_SIZE = 18; // px
const GRID_COLOR = "#CCCCCC";
const TASK_COLOR = "#FFFFFF";

let optim = Optim.new();
let maintenance;

// create vertex shader program (how the vertices are treated)
const vertCode = 
    'attribute vec3 position;' +
    'uniform mat4 Pmatrix;'+
    'uniform mat4 Vmatrix;'+
    'uniform mat4 Mmatrix;'+
    'void main(void) {' +
    'gl_Position = Pmatrix*Vmatrix*Mmatrix*vec4(position, 1.);'+
    '}';
const vertShader = gl_planning.createShader(gl_planning.VERTEX_SHADER);
gl_planning.shaderSource(vertShader, vertCode);
gl_planning.compileShader(vertShader);

// create fragment shader program (how the pixel are treated)
const fragCode =
    'precision mediump float;' +
    'void main(void) {' +
    'gl_FragColor = vec4(0.0, 0.0, 0.0, 1.0);' +
    '}';
const fragShader = gl_planning.createShader(gl_planning.FRAGMENT_SHADER);
gl_planning.shaderSource(fragShader, fragCode);
gl_planning.compileShader(fragShader);

// Create gl program by linking compiled shaders, then load it.
const shaderProgram = gl_planning.createProgram();
gl_planning.attachShader(shaderProgram, vertShader);
gl_planning.attachShader(shaderProgram, fragShader);
gl_planning.linkProgram(shaderProgram);
gl_planning.useProgram(shaderProgram);


const loadPlanning = () => {
    const grid_size = 1.8;
    const grid_row_start = -0.9;
    const grid_column_start = -0.9;
    const grid_width = maintenance.ndays();
    const grid_height = 5;
    const grid_w_step = grid_size / grid_width;
    const grid_h_step = grid_size / grid_height;
    const grid_vertices = [];
    for(var row = 0; row <= grid_height; ++row) {
        let x_start = grid_column_start;
        let x_end = grid_column_start + grid_size;
        let y = grid_row_start + grid_h_step*row;
        grid_vertices[grid_vertices.length] = x_start;
        grid_vertices[grid_vertices.length] = y;
        grid_vertices[grid_vertices.length] = 0;
        grid_vertices[grid_vertices.length] = x_end;
        grid_vertices[grid_vertices.length] = y;
        grid_vertices[grid_vertices.length] = 0;
    }
    for(var column = 0; column <= grid_width; ++column) {
        let y_start = grid_row_start;
        let y_end = grid_row_start + grid_size;
        let x = grid_column_start + grid_w_step*column;
        grid_vertices[grid_vertices.length] = x;
        grid_vertices[grid_vertices.length] = y_start;
        grid_vertices[grid_vertices.length] = 0;
        grid_vertices[grid_vertices.length] = x;
        grid_vertices[grid_vertices.length] = y_end;
        grid_vertices[grid_vertices.length] = 0;
    }
    // add all points of the grid
    for(var row = 0; row <= grid_height; ++row) {
        let y = grid_row_start + grid_h_step*row;
        for(var column = 0; column <= grid_width; ++column) {
            let x = grid_column_start + grid_w_step*column;
            grid_vertices[grid_vertices.length] = x;
            grid_vertices[grid_vertices.length] = y;
            grid_vertices[grid_vertices.length] = 0;
        }
    }
    //const createSquares = (indexes) => {
        //let squares = [];
        //let offset = 2*(grid_width + grid_height + 2);
        //indexes.forEach((index) => {
            //index = index + Math.floor(index / grid_width);

            //squares.push(offset + index,
                //offset + index+1,
                //offset + index+grid_width+1,
                //offset + index+1,
                //offset + index+(grid_width+1),
                //offset + index+(grid_width+1)+1);
        //});
        //return squares;
    //};

    // gl buffer for grid
    const gl_grid_vertices = gl_planning.createBuffer();
    gl_planning.bindBuffer(gl_planning.ARRAY_BUFFER, gl_grid_vertices);
    gl_planning.bufferData(gl_planning.ARRAY_BUFFER, new Float32Array(grid_vertices), gl_planning.STATIC_DRAW);
    gl_planning.bindBuffer(gl_planning.ARRAY_BUFFER, null);

    // Associate the desired buffer objects to shader programs
    const Pmatrix = gl_planning.getUniformLocation(shaderProgram, "Pmatrix");
    const Vmatrix = gl_planning.getUniformLocation(shaderProgram, "Vmatrix");
    const Mmatrix = gl_planning.getUniformLocation(shaderProgram, "Mmatrix");
    gl_planning.bindBuffer(gl_planning.ARRAY_BUFFER, gl_grid_vertices);

    /*========================= MATRIX ========================= */

    function get_projection(angle, a, zMin, zMax) {
        var ang = Math.tan((angle*.5)*Math.PI/180);//angle*.5
        return [
            0.5/ang, 0 , 0, 0,
            0, 0.5*a/ang, 0, 0,
            0, 0, -(zMax+zMin)/(zMax-zMin), -1,
            0, 0, (-2*zMax*zMin)/(zMax-zMin), 0
        ];
    }


    const full_h_step = 2.0 / grid_height;
    const scale = 1.0 / (grid_h_step*grid_height/full_h_step);
    var proj_matrix = get_projection(10.0, planning.width/planning.height, 1, 6);
    //var proj_matrix = [1,0,0,0, 0,1,0,0, 0,0,1,0, 0,0,0,1];
    var mov_matrix = [1,0,0,0, 0,1,0,0, 0,0,1,0, 0,0,0,1];
    var view_matrix = [1,0,0,0, 0,scale,0,0, 0,0,1,0, 0,0,0,1];

    //translating z
    view_matrix[14] = view_matrix[14]-6; //zoom

    const position = gl_planning.getAttribLocation(shaderProgram, "position")
    gl_planning.vertexAttribPointer(position, 3, gl_planning.FLOAT, false, 0, 0);
    gl_planning.enableVertexAttribArray(position);
    const gl_square_vertices = gl_planning.createBuffer();
    var time_old = 0;

    // gl buffer for one square
    gl_planning.bindBuffer(gl_planning.ELEMENT_ARRAY_BUFFER, gl_square_vertices);

    // Draw the required objects
    gl_planning.enable(gl_planning.DEPTH_TEST);
    gl_planning.depthFunc(gl_planning.LEQUAL);
    gl_planning.clearColor(1.0, 1.0, 1.0, 1.0);
    gl_planning.clearDepth(1.0);
    gl_planning.viewport(0, 0, planning.width, planning.height);
    gl_planning.clear(gl_planning.COLOR_BUFFER_BIT | gl_planning.DEPTH_BUFFER_BIT);

    gl_planning.uniformMatrix4fv(Pmatrix, false, proj_matrix);
    gl_planning.uniformMatrix4fv(Vmatrix, false, view_matrix);
    gl_planning.uniformMatrix4fv(Mmatrix, false, mov_matrix);

    gl_planning.drawArrays(gl_planning.LINES, 0, 2*(grid_width + grid_height + 2));
};

//var request = new XMLHttpRequest();
//request.open('GET', 'http://192.168.56.3:8000/test', true);
//request.onload = function() {
//console.log("OK");
//alert(request.responseText);
//}

//request.send();
//
//request.setRequestHeader("Content-Type", "application.json");
//request.onload = function() {
//console.log("OK");
//console.log(request.responseText);
//}
//var data = JSON.stringify({value: 10});
//console.log(data);
//request.send(data);

input_file.addEventListener("change", event => {
    console.log("load file");
});

var start = new Date();
send_file.addEventListener("click", event => {
    const selectedFile = input_file.files[0];
    const fd = new FormData();
    fd.append("file", selectedFile);
    var request = new XMLHttpRequest();
    request.overrideMimeType("application/octet-stream")
    request.responseType = 'arraybuffer';
    request.open('POST', 'http://192.168.56.3:8000/optim/new', true);
    request.onload = function() {
        const data = Uuid.from_bytes(new Uint8Array(request.response));
        //maintenance = summary;
        //const summary_table = document.getElementById('summary-table');
        //summary_table.style = '';
        //const days = document.getElementById('ndays');
        //days.innerHTML = "" + maintenance.ndays();
        //const interventions = document.getElementById('ninterventions');
        //interventions.innerHTML = "" + maintenance.ninterventions();
        //const resources = document.getElementById('nresources');
        //resources.innerHTML = "" + maintenance.nresources();
        //const scenarios = document.getElementById('nscenarios');
        //scenarios.innerHTML = "" + maintenance.nscenarios();
        //loadPlanning();
    }
    request.send(fd);
});

//var start = new Date();
//send_file.addEventListener("click", event => {
    //const selectedFile = input_file.files[0];
    //const fd = new FormData();
    //fd.append("file", selectedFile);
    //var request = new XMLHttpRequest();
    //request.overrideMimeType("application/octet-stream")
    //request.responseType = 'arraybuffer';
    //request.open('POST', 'http://192.168.56.3:8000/optim', true);
    //request.onload = function() {
        //const summary = MaintenanceSummary.from_bytes(new Uint8Array(request.response));
        //maintenance = summary;
        //const summary_table = document.getElementById('summary-table');
        //summary_table.style = '';
        //const days = document.getElementById('ndays');
        //days.innerHTML = "" + maintenance.ndays();
        //const interventions = document.getElementById('ninterventions');
        //interventions.innerHTML = "" + maintenance.ninterventions();
        //const resources = document.getElementById('nresources');
        //resources.innerHTML = "" + maintenance.nresources();
        //const scenarios = document.getElementById('nscenarios');
        //scenarios.innerHTML = "" + maintenance.nscenarios();
        //loadPlanning();
    //}
    //request.send(fd);
//});

send_file_2.addEventListener("click", event => {
    const selectedFile = input_file.files[0];
    const fd = new FormData();
    fd.append("file", selectedFile);
    var request = new XMLHttpRequest();
    request.open('POST', 'http://192.168.56.3:8000/optim-json', true);
    request.onload = function() {
        console.log(request.responseText);
    }
    request.send(fd);
});


//var start = new Date();
//send_file.addEventListener("click", event => {
//console.log("Send");
//const selectedFile = input_file.files[0];
//console.log(selectedFile);
//console.log(selectedFile.webkitRelativePath);
//const reader = new FileReader();
//reader.onload = function () {
////JSON.parse(reader.result);
////console.log("JSON: ok");
//const data = new Uint8Array(reader.result);
//optim.load_from_bytes(data);
////data.set([]);
//const end = new Date();
//const diff = new Date(end - start);
//console.log(diff.getSeconds() +"s");
//console.log(optim.is_loaded());
//console.log(optim.ninterventions());
//start = new Date();
//loadPlanning();
//var end2 = new Date();
//const diff2 = new Date(end2 - start);
//console.log(diff2.getMilliseconds() +"ms");
//};
////reader.readAsText(selectedFile);
//if(optim.is_loaded()) {
//optim = Optim.new();
//}
//start = new Date();
//reader.readAsArrayBuffer(selectedFile);
////const fd = new FormData();
////fd.append("file", selectedFile);

////var request = new XMLHttpRequest();
////request.open('POST', 'http://192.168.56.3:8000/optim2', true);
////request.onload = function() {
////console.log("OK");
////console.log(request.responseText);
////}
////console.log("Send file: " + selectedFile);
////request.send(fd);
//});

