function main() {
    console.log("Hello, world!");
}

class Point {
    constructor(x, y) {
        this.x = x;
        this.y = y;
    }
    
    distance(other) {
        return Math.sqrt((this.x - other.x)**2 + (this.y - other.y)**2);
    }
}

main();