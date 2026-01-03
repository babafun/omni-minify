// START OF BLOATED JS
/*
  Welcome to the bloated file.
  Everything here is inefficient, verbose, and repetitive.
*/

function addNumbers(a, b) {
        return a + b; // simple addition, but we write it fancy
}

function multiplyNumbers(a, b) { return a * b; }

function uselessLoop() {
    for (let i = 0; i < 1000; i++) {
        // doing absolutely nothing
        let tmp = i * 2;
        tmp = tmp / 2;
    }
}

// useless variables
var x = 42;
var y = "hello";
var z = [1,2,3,4,5,6,7,8,9,10];

function redundantFunctions() {
    function inner1() { return 1+1; }
    function inner2() { return 2+2; }
    function inner3() { return 3+3; }
    return inner1() + inner2() + inner3();
}

// deeply nested useless object
var megaObject = {
    level1: {
        level2: {
            level3: {
                a: 1,
                b: 2,
                c: {
                    x: 10,
                    y: 20,
                    z: [1,2,3,4,5]
                }
            },
            uselessArray: [1,2,3,4,5,6,7,8,9,10,11,12]
        }
    }
};

// repeated constants
const REPEAT_ME = 1;
const REPEAT_ME2 = 1;
const REPEAT_ME3 = 1;
const REPEAT_ME4 = 1;
const REPEAT_ME5 = 1;

// redundant conditionals
function checkCondition(a) {
    if(a > 0) { return true; } else { return false; }
}

// huge unused function
function megaFunction() {
    var sum = 0;
    for(var i=0;i<100;i++){
        sum += i;
    }
    function nested() {
        return sum * 2;
    }
    return nested();
}

// IIFE that does nothing
(function(){
    var temp = 12345;
    console.log("IIFE running but useless");
})();

// more dead code
var neverUsed = [1,2,3,4,5,6,7,8,9,10];
var alsoUnused = {
    a:1,
    b:2,
    c:3,
    d:4
};

function helloWorld() {
    console.log("Hello, world!");
    return "Hello";
}

// END OF BLOATED JS
