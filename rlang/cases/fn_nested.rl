

fn nested(a) {
    if (a < 3) {
        if (a > 1) return a;
    }

    {
    a = a + 2;
    return a;
    }

    return -1;
}

print nested(2);
print nested(1);