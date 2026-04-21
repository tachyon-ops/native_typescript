function Array(len) {
    this.length = len;
}

function array_push(...args) {
    for (let i = 0; i < args.length; i++) {
        this[this.length] = args[i];
        this.length = this.length + 1;
    }
    return this.length;
}
Array.prototype.push = array_push;

function array_pop() {
    if (this.length === 0) return undefined;
    let val = this[this.length - 1];
    this.length = this.length - 1;
    return val;
}
Array.prototype.pop = array_pop;

function array_shift() {
    if (this.length === 0) return undefined;
    let val = this[0];
    for (let i = 1; i < this.length; i++) {
        this[i - 1] = this[i];
    }
    this.length = this.length - 1;
    return val;
}
Array.prototype.shift = array_shift;

function array_unshift(...args) {
    for (let i = this.length - 1; i >= 0; i--) {
        this[i + args.length] = this[i];
    }
    for (let i = 0; i < args.length; i++) {
        this[i] = args[i];
    }
    this.length = this.length + args.length;
    return this.length;
}
Array.prototype.unshift = array_unshift;

function array_join(sep) {
    if (sep === undefined) sep = ',';
    let res = "";
    for (let i = 0; i < this.length; i++) {
        let elem = this[i];
        if (Array.isArray(elem)) {
            res = res + elem.join();
        } else if (elem === undefined || elem === null) {
            res = res + "";
        } else {
            res = res + elem;
        }
        if (i < this.length - 1) res = res + sep;
    }
    return res;
}
Array.prototype.join = array_join;

function array_map(fn) {
    let res = [];
    for (let i = 0; i < this.length; i++) {
        res.push(fn(this[i], i, this));
    }
    return res;
}
Array.prototype.map = array_map;

function array_filter(fn) {
    let res = [];
    for (let i = 0; i < this.length; i++) {
        if (fn(this[i], i, this)) res.push(this[i]);
    }
    return res;
}
Array.prototype.filter = array_filter;

function array_reduce(fn, init) {
    let acc = init;
    let start = 0;
    if (init === undefined) {
        acc = this[0];
        start = 1;
    }
    for (let i = start; i < this.length; i++) {
        acc = fn(acc, this[i], i, this);
    }
    return acc;
}
Array.prototype.reduce = array_reduce;

function array_reduceRight(fn, init) {
    let acc = init;
    let start = this.length - 1;
    if (init === undefined) {
        acc = this[start];
        start = start - 1;
    }
    for (let i = start; i >= 0; i--) {
        acc = fn(acc, this[i], i, this);
    }
    return acc;
}
Array.prototype.reduceRight = array_reduceRight;

function array_forEach(fn) {
    for (let i = 0; i < this.length; i++) {
        fn(this[i], i, this);
    }
}
Array.prototype.forEach = array_forEach;

function array_find(fn) {
    for (let i = 0; i < this.length; i++) {
        if (fn(this[i], i, this)) return this[i];
    }
    return undefined;
}
Array.prototype.find = array_find;

function array_findIndex(fn) {
    for (let i = 0; i < this.length; i++) {
        if (fn(this[i], i, this)) return i;
    }
    return -1;
}
Array.prototype.findIndex = array_findIndex;

function array_findLast(fn) {
    for (let i = this.length - 1; i >= 0; i--) {
        if (fn(this[i], i, this)) return this[i];
    }
    return undefined;
}
Array.prototype.findLast = array_findLast;

function array_some(fn) {
    for (let i = 0; i < this.length; i++) {
        if (fn(this[i], i, this)) return true;
    }
    return false;
}
Array.prototype.some = array_some;

function array_every(fn) {
    for (let i = 0; i < this.length; i++) {
        if (!fn(this[i], i, this)) return false;
    }
    return true;
}
Array.prototype.every = array_every;

function array_includes(val) {
    for (let i = 0; i < this.length; i++) {
        let v = this[i];
        if (v === val) return true;
        if (typeof v === 'number' && typeof val === 'number') {
            if (v !== v && val !== val) return true;
        }
    }
    return false;
}
Array.prototype.includes = array_includes;

function array_indexOf(val) {
    for (let i = 0; i < this.length; i++) {
        if (this[i] === val) return i;
    }
    return -1;
}
Array.prototype.indexOf = array_indexOf;

function array_lastIndexOf(val) {
    for (let i = this.length - 1; i >= 0; i--) {
        if (this[i] === val) return i;
    }
    return -1;
}
Array.prototype.lastIndexOf = array_lastIndexOf;

function array_slice(start, end) {
    if (start === undefined) start = 0;
    if (start < 0) start = this.length + start;
    if (end === undefined) end = this.length;
    if (end < 0) end = this.length + end;
    let res = [];
    for (let i = start; i < end; i++) {
        res.push(this[i]);
    }
    return res;
}
Array.prototype.slice = array_slice;

function array_splice(start, deleteCount, ...items) {
    if (start < 0) start = this.length + start;
    if (deleteCount === undefined) deleteCount = this.length - start;
    let res = [];
    for (let i = 0; i < deleteCount; i++) {
        res.push(this[start + i]);
    }
    let diff = items.length - deleteCount;
    if (diff > 0) {
        for (let i = this.length - 1; i >= start + deleteCount; i--) {
            this[i + diff] = this[i];
        }
    } else if (diff < 0) {
        for (let i = start + deleteCount; i < this.length; i++) {
            this[i + diff] = this[i];
        }
    }
    for (let i = 0; i < items.length; i++) {
        this[start + i] = items[i];
    }
    this.length = this.length + diff;
    return res;
}
Array.prototype.splice = array_splice;

function array_concat(...args) {
    let res = this.slice();
    for (let i = 0; i < args.length; i++) {
        if (Array.isArray(args[i])) {
            for (let j = 0; j < args[i].length; j++) {
                res.push(args[i][j]);
            }
        } else {
            res.push(args[i]);
        }
    }
    return res;
}
Array.prototype.concat = array_concat;

function array_sort(fn) {
    if (fn === undefined) {
        fn = function(a, b) {
            let strA = "" + a;
            let strB = "" + b;
            if (strA < strB) return -1;
            if (strA > strB) return 1;
            return 0;
        };
    }
    for (let i = 0; i < this.length; i++) {
        for (let j = 0; j < this.length - 1; j++) {
            if (fn(this[j], this[j + 1]) > 0) {
                let tmp = this[j];
                this[j] = this[j + 1];
                this[j + 1] = tmp;
            }
        }
    }
    return this;
}
Array.prototype.sort = array_sort;

function array_reverse() {
    let left = 0;
    let right = this.length - 1;
    while (left < right) {
        let tmp = this[left];
        this[left] = this[right];
        this[right] = tmp;
        left = left + 1;
        right = right - 1;
    }
    return this;
}
Array.prototype.reverse = array_reverse;

function array_fill(val, start, end) {
    if (start === undefined) start = 0;
    if (end === undefined) end = this.length;
    for (let i = start; i < end; i++) {
        this[i] = val;
    }
    return this;
}
Array.prototype.fill = array_fill;

function array_copyWithin(target, start, end) {
    if (end === undefined) end = this.length;
    let len = end - start;
    let tmp = [];
    for (let i = 0; i < len; i++) {
        tmp.push(this[start + i]);
    }
    for (let i = 0; i < len; i++) {
        this[target + i] = tmp[i];
    }
    return this;
}
Array.prototype.copyWithin = array_copyWithin;

function array_flat(depth) {
    if (depth === undefined) depth = 1;
    let res = [];
    for (let i = 0; i < this.length; i++) {
        if (Array.isArray(this[i]) && depth > 0) {
            let flatItem = this[i].flat(depth - 1);
            for (let j = 0; j < flatItem.length; j++) {
                res.push(flatItem[j]);
            }
        } else {
            res.push(this[i]);
        }
    }
    return res;
}
Array.prototype.flat = array_flat;

function array_flatMap(fn) {
    let res = [];
    for (let i = 0; i < this.length; i++) {
        let mapped = fn(this[i], i, this);
        if (Array.isArray(mapped)) {
            for (let j = 0; j < mapped.length; j++) {
                res.push(mapped[j]);
            }
        } else {
            res.push(mapped);
        }
    }
    return res;
}
Array.prototype.flatMap = array_flatMap;

function array_at(index) {
    if (index < 0) index = this.length + index;
    return this[index];
}
Array.prototype.at = array_at;

function array_keys() {
    let res = [];
    for (let i = 0; i < this.length; i++) {
        res.push(i);
    }
    return res;
}
Array.prototype.keys = array_keys;

function array_values() {
    let res = [];
    for (let i = 0; i < this.length; i++) {
        res.push(this[i]);
    }
    return res;
}
Array.prototype.values = array_values;

function array_entries() {
    let res = [];
    for (let i = 0; i < this.length; i++) {
        res.push([i, this[i]]);
    }
    return res;
}
Array.prototype.entries = array_entries;

function Array_from(iterable, mapFn) {
    let res = [];
    if (Array.isArray(iterable) || iterable.length !== undefined) {
        for (let i = 0; i < iterable.length; i++) {
            let val = iterable[i];
            if (mapFn) val = mapFn(val, i);
            res.push(val);
        }
    } else {
        if (typeof iterable === 'string') {
            for (let i = 0; i < iterable.length; i++) {
                let val = iterable[i];
                if (mapFn) val = mapFn(val, i);
                res.push(val);
            }
        }
    }
    return res;
}
Array.from = Array_from;

function Array_of(...args) {
    return args;
}
Array.of = Array_of;

function Array_isArray(obj) {
    return obj && typeof obj === 'object' && obj.length !== undefined && obj.push !== undefined;
}
Array.isArray = Array_isArray;

function Float64Array(data) {
    if (typeof data === 'number') {
        this.length = data;
        for (let i = 0; i < data; i++) {
            this[i] = 0;
        }
    } else {
        this.length = data.length;
        for (let i = 0; i < data.length; i++) {
            this[i] = data[i];
        }
    }
}
Float64Array.prototype.reduce = array_reduce;

function Int32Array(data) {
    if (typeof data === 'number') {
        this.length = data;
        for (let i = 0; i < data; i++) {
            this[i] = 0;
        }
    } else {
        this.length = data.length;
        for (let i = 0; i < data.length; i++) {
            this[i] = data[i];
        }
    }
}
Int32Array.prototype.reduce = array_reduce;

function Uint8Array(data) {
    if (typeof data === 'number') {
        this.length = data;
        for (let i = 0; i < data; i++) {
            this[i] = 0;
        }
    } else {
        this.length = data.length;
        for (let i = 0; i < data.length; i++) {
            this[i] = data[i];
        }
    }
}
Uint8Array.prototype.reduce = array_reduce;
