import { Component, OnInit, Inject } from '@angular/core';
import { ActivatedRoute, Router } from '@angular/router';
import { OmEnvironment, GenericApi } from '../rest.service';

@Component({
    selector: 'app-om_environment',
    template: `env`,
})
export class OmEnvironmentListComponent implements OnInit {

    list: OmEnvironment[] = [];

    constructor(
        @Inject("OmEnvironmentService") private rest: GenericApi<OmEnvironment>,
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
