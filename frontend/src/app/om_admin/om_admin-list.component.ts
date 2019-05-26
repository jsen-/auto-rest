import { Component, OnInit, Inject } from '@angular/core';
import { OmAdmin, GenericApi } from '../rest.service';
import { ActivatedRoute, Router } from '@angular/router';
import { OmAdminAddComponent } from "./om_admin-add.component";

@Component({
    selector: 'app-om_admin',
    template: `
<div *ngIf="!adding">
    <a (click)="adding = true">add</a>
</div>
<div *ngIf="adding">
    <app-om_admin-add (done)="update_list()"></app-om_admin-add>
</div>
<ul class="products">
    <li *ngFor="let item of list; let i=index;">
        <!--a routerLink="/product-details/{{p._id}}"-->
        <span class="badge">{{i+1}}</span> {{item.name}} {{item.mail}} {{item.sm_login}}
        <button class="delete" title="delete admin" (click)="delete(item)">x</button>
    </li>
</ul>`,
})
export class OmAdminListComponent implements OnInit {

    list: OmAdmin[] = [];
    adding = false;

    constructor(@Inject("OmAdminService") private rest: GenericApi<OmAdmin>,
        private route: ActivatedRoute,
        private router: Router,
    ) { }

    ngOnInit() {
        this.update_list();
    }

    update_list() {
        this.rest.get_all().subscribe((data) => this.list = data);
    }
    delete(id: number) {
        this.rest.delete(id)
            .subscribe(_ => this.update_list(), console.error);
    }

}
