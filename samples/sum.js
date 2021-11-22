function sum(n) {
    for (var i=0, s=0; i<n; i++)
        s += i;
    return s;
}

__console_log(sum(3));