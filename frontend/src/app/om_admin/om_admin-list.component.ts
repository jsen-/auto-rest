import { Component, OnInit, Inject } from '@angular/core';
import { OmAdmin, GenericApi } from '../rest.service';
import { ActivatedRoute, Router } from '@angular/router';
import { OmAdminAddComponent } from "./om_admin-add.component";

@Component({
    selector: 'app-om_admin',
    styles: [`
#error {
    position: fixed;
    width: 100%;
    opacity: 0.9;
    padding: 20px;
    background-color: red;
    color: yellow;
}
`],
    template: `
<div *ngIf="error" id="error">{{error}}</div>
<app-om_admin-add (done)="refresh()" (error)="show_error($event)"></app-om_admin-add>
<ul class="products">
    <li *ngFor="let item of list; let i=index;">
        <span class="badge">{{i+1}}</span> {{item.name}} {{item.mail}} {{item.sm_login}}
        <button class="delete" title="delete admin" (click)="delete(item)">x</button>
    </li>
</ul>`,
})
export class OmAdminListComponent {

    error?: string = "";
    error_timeout?: number;

    list: OmAdmin[] = [];

    constructor(@Inject("OmAdminService") private rest: GenericApi<OmAdmin>) {
        this.refresh();
    }

    refresh() {
        this.rest.get_all().subscribe(
            (data) => this.list = data,
            (err) => this.show_error(err)
        );
    }

    show_error(message: string) {
        this.error = message;
        clearTimeout(this.error_timeout);
        this.error_timeout = setTimeout(() => this.error = "", 3000);
    }

    delete(item: OmAdmin) {
        if (window.confirm(`Are sure you want to delete "${item.name}" ?`)) {
            this.rest.delete(item.id)
                .subscribe(
                    _ => this.refresh(),
                    (err) => this.show_error(err)
                );
        }
    }

}
