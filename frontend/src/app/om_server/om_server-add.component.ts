import { Component, OnInit, Inject } from '@angular/core';
import { ActivatedRoute, Router } from '@angular/router';
import { GenericApi, OmServer } from '../rest.service';

@Component({
    selector: 'app-om_server-add',
    template: `
<form (submit)="submit">
    <label for="">
</form>`,
})
export class OmServerAddComponent implements OnInit {

    constructor(
        @Inject("OmServerService") private rest: GenericApi<OmServer>,
        private route: ActivatedRoute,
        private router: Router,
    ) { }

    ngOnInit() {

    }

    submit() {
        // this.rest.add(id)
        //     .subscribe(_ => this.update_list(), console.error);
    }
}
