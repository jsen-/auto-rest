import { Component, OnInit, Inject } from '@angular/core';
import { OmServer, GenericApi } from '../rest.service';
import { ActivatedRoute, Router } from '@angular/router';

@Component({
    selector: 'app-om_server',
    template: `
    <h2>OM Servers</h2>

    <div>
      <a routerLink="/om_servers/add">Add</a>
    </div>

    <ul class="products">
      <li *ngFor="let item of list; let i=index;">
        <!--a routerLink="/product-details/{{p._id}}"-->
        <span class="badge">{{i+1}}</span> {{item.fqdn}}
        <button class="delete" title="delete server" (click)="delete(item)">x</button>
      </li>
    </ul>`,
    // styleUrls: ['./om_server.component.css']
})
export class OmServerListComponent implements OnInit {

    list: OmServer[] = [];

    constructor(@Inject("OmServerService") private rest: GenericApi<OmServer>,
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
