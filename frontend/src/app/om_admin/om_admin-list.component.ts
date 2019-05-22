import { Component, OnInit, Inject } from '@angular/core';
import { OmAdmin, GenericApi } from '../rest.service';
import { ActivatedRoute, Router } from '@angular/router';

@Component({
    selector: 'app-om_admin',
    template: `admin`,
})
export class OmAdminListComponent implements OnInit {

    list: OmAdmin[] = [];

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
