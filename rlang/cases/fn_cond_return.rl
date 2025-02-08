
fn cond_ret(a) {
    if (a < 0) return 0;
    return a - 1;
}

print cond_ret(4);
print cond_ret(3);
print cond_ret(2);
print cond_ret(-1);