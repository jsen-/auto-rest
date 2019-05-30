import { Injectable, Inject } from '@angular/core';
import { HttpClient, HttpHeaders, HttpErrorResponse } from '@angular/common/http';
import { Observable, of } from 'rxjs';
import { map, catchError, tap } from 'rxjs/operators';
import { Option, None, Some } from "./utils/option";

const httpOptions = {
    headers: new HttpHeaders({
        'Content-Type': 'application/json'
    })
};

type Id = { id: number };
export type SansId<T extends Id> = Pick<T, Exclude<keyof T, "id">>;
type Json<T extends Id> = SansId<T> & { id: number };

function extractData<T extends Id>(res: T[]): T[] {
    return res || [];
}
function map_extract<T extends Id>() {
    return map<any, T[]>(extractData);
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



@Injectable({
    providedIn: 'root',
})
export class GenericApi<T extends Id> {
    constructor(private http: HttpClient, private path: string, private api_name: string) { }

    public get_all(): Observable<T[]> {
        return this.http.get(this.path).pipe(map_extract())
    }
    // public get(id: number): Observable<T> {
    //     return this.http.get(`${this.path}/${id}`).pipe(map_extract());
    // }
    public add(t: SansId<T>): Observable<T> {
        return this.http.post<T>(`${this.path}`, JSON.stringify(t), httpOptions)
            .pipe(
                tap((t) => console.log(`added ${this.api_name} w/ id=${t.id}`)),
                // catchError(handleError<T>(`add ${this.api_name}`))
            );
    }
    // public update(id: number, t: T): Observable<T> {
    //     return this.http.put<T>(`${this.path}/${id}`, JSON.stringify(t), httpOptions)
    //         .pipe(
    //             tap(_ => console.log(`updated ${this.api_name} id=${id}`)),
    //             catchError(handleError<T>(`update ${this.api_name}`))
    //         );
    // }
    public delete(id: number): Observable<T> {
        return this.http.delete<T>(`${this.path}/${id}`, httpOptions)
            .pipe(
                tap(_ => console.log(`deleted ${this.api_name} id=${id}`)),
                // catchError(handleError<T>(`delete ${this.api_name}`))
            );
    }
}

type Ref<T> = { [K in keyof T]: T[K] };
type Obj<T> = { [K in keyof T]: T[K] };

export class Product {
    id!: number;
    name!: string;
    desc!: string;
    price!: number;
    updated_at?: Date;

    private constructor() { }
    static new(props: Obj<Product>): Product {
        return Object.assign(new Product(), props);
    }
}

export class OmAdmin {
    id!: number;
    name!: string;
    mail!: string;
    sm_login!: string;
}

export class OmEnvironment {
    id!: number;
    name!: string;
    om_admin!: Option<Ref<OmAdmin>>;
}

export class OmServer {
    id!: number;
    fqdn!: string;
    alias!: Option<string>;
    om_environment!: Ref<OmEnvironment>
    type!: number; // 1:primary, 0:secondary,-1:pooling

    private constructor() { }

    static new(props: Obj<OmServer>): OmServer {
        return Object.assign(new OmServer(), props);
    }
}
