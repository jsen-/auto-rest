import { Injectable } from '@angular/core';
import { HttpClient, HttpHeaders, HttpErrorResponse } from '@angular/common/http';
import { Observable, of } from 'rxjs';
import { map, catchError, tap } from 'rxjs/operators';
import { Option, None, Some } from "./utils/option";

const endpoint = `${window.location.protocol}//${window.location.host}/api/v1/`;
const httpOptions = {
    headers: new HttpHeaders({
        'Content-Type': 'application/json'
    })
};

function extractData(res) {
    return res || {};
}
function map_extract<T>() {
    return map<Response, T>(extractData);
}

function handleError<T>(operation = 'operation', result?: T) {
    return (error: any): Observable<T> => {
        // TODO: send the error to remote logging infrastructure
        console.error(error); // log to console instead
        // TODO: better job of transforming error for user consumption
        console.log(`${operation} failed: ${error.message}`);
        // Let the app keep running by returning an empty result.
        return of(result as T);
    };
}

export class GenericApi<T extends { id: Option<number> }> {
    constructor(private http: HttpClient, private path: string, private api_name: string) {
    }
    public get_all(): Observable<T> {
        return this.http.get(this.path).pipe(map_extract<T>())
    }
    public get(id: number): Observable<T> {
        return this.http.get(`${this.path}/${id}`).pipe(map_extract<T>());
    }
    public add(t: T): Observable<T> {
        return this.http.post<T>(`${this.path}`, JSON.stringify(t), httpOptions)
            .pipe(
                tap((t) => console.log(`added ${this.api_name} w/ id=${t.id}`)),
                catchError(handleError<T>(`add ${this.api_name}`))
            );
    }
    public update(id, t): Observable<T> {
        return this.http.put<T>(`${this.path}/${id}`, JSON.stringify(t), httpOptions)
            .pipe(
                tap(_ => console.log(`updated ${this.api_name} id=${id}`)),
                catchError(handleError<T>(`update ${this.api_name}`))
            );
    }
    public delete(id): Observable<T> {
        return this.http.delete<T>(`${this.path}/${id}`, httpOptions)
            .pipe(
                tap(_ => console.log(`deleted ${this.api_name} id=${id}`)),
                catchError(handleError<T>(`delete ${this.api_name}`))
            );
    }
}

export class Product {
    id: Option<number>;
    name: string;
    desc: string;
    price: number;
    updated_at?: Date;

    static new(): Product {
        return {
            id: None.new(),
            name: "",
            desc: "",
            price: 0,
        }
    }
}


@Injectable({
    providedIn: 'root'
})
export class RestService {
    product: GenericApi<Product>;
    constructor(private http: HttpClient) {
        this.product = new GenericApi<Product>(http, `${endpoint}/om_server`, "product");
    }
}
