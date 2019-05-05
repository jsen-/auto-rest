export interface IOption<T> {
    is_some(): this is Some<T>;
    is_none(): this is None<T>;
    map<T2>(mapper: (value: T) => T2): Option<T2>;
    or_else(mapper: ()=> T): Option<T>;
    and_then<U>(mapper: (value: T) => Option<U>): Option<U>;
    unwrap_or(value: T): T;
    unwrap_or_else(mapper: () => T): T;
    unwrap(): T;
    unwrap_none(): void;
    expect(message: string): T;
    iter(): Iterable<T>;
    toString(): string;
}

export class Some<T> implements IOption<T> {

    static new<T>(value: T): Some<T> {
        return new Some<T>(value);
    }

    // TODO: find out why the bang is needed
    protected _val!: T;

    constructor(value: T) {
        if (this instanceof Some) {
            this._val = value;
            return this;
        }
        return new Some(value);
    }

    is_some(): this is Some<T> {
        return true;
    }
    is_none(): this is None<T> {
        return false;
    }
    map<T2>(mapper: (value: T) => T2): Option<T2> {
        return new Some(mapper(this._val));
    }
    or_else(mapper: ()=> T): Option<T> {
        return this;
    }
    and_then<U>(mapper: (value: T) => Option<U>): Option<U> {
        return mapper(this._val);
    }
    unwrap_or(_: T): T {
        return this._val;
    }
    unwrap_or_else(_: () => T): T {
        return this._val;
    }
    unwrap(): T {
        return this._val;
    }
    unwrap_none(): void {
        throw new TypeError(`unwrap_none called on Some Option variant`);
    }
    expect(_: string): T {
        return this._val;
    }
    iter(): Iterable<T> {
        let called = false;
        const value = this._val;
        return {
            [Symbol.iterator](): Iterator<T> {
                return {
                    next(_?: any): IteratorResult<T> {
                        return called
                            ? { value: undefined as any as T, done: true }
                            : (called = true, { value, done: false });
                    }
                };
            }
        };
    }
    toString() {
        return `Ok { ${this._val} }`;
    }
}

export class None<T> implements IOption<T> {
    static new<T>(): None<T> {
        return new None<T>();
    }

    constructor() {
        if (this instanceof None) {
            return this;
        }
        return new None();
    }

    is_some(): this is Some<T> {
        return false;
    }
    is_none(): this is None<T> {
        return true;
    }
    map<T2>(_: (value: T) => T2): Option<T2> {
        return this as any;
    }
    and_then<U>(_: (value: T) => Option<U>): Option<U> {
        return this as any;
    }
    or_else(mapper: () => T): Option<T> {
        return new Some(mapper());
    }
    unwrap_or(value: T): T {
        return value;
    }
    unwrap_or_else(mapper: () => T): T {
        return mapper();
    }
    unwrap(): T {
        throw new TypeError(`unwrap called on None Option variant`);
    }
    unwrap_none(): void {
    }
    expect(message: string): T {
        throw new TypeError(message);
    }
    iter(): Iterable<T> {
        return {
            [Symbol.iterator](): Iterator<T> {
                return {
                    next(_?: any): IteratorResult<T> {
                        return { value: undefined as any as T, done: true };
                    }
                };
            }
        };
    }
    toString() {
        return `None`;
    }
}

export type Option<T> = Some<T> | None<T>;
