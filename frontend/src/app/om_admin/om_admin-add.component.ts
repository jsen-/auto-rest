import { Component, OnInit, Inject } from '@angular/core';
import { ActivatedRoute, Router } from '@angular/router';
import { GenericApi, OmAdmin } from '../rest.service';

@Component({
    selector: 'app-om_server-add',
    template: `
<form (submit)="submit()">
    <label for="name">name</label><input type="text" name="name" id="name"><br>
    <label for="mail">mail</label><input type="text" name="mail" id="mail"><br>
    <label for="sm_login">sm_login</label><input type="text" name="sm_login" id="sm_login"><br>
</form>`,
})
export class OmAdminAddComponent implements OnInit {

    constructor(
        @Inject("OmAdminService") private rest: GenericApi<OmAdmin>,
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
