
fn thrice(func) {
    for (var i = 1; i <= 3; i = i + 1) {
        func(i);
    }
}

thrice(fn (a) {
    print a;
});