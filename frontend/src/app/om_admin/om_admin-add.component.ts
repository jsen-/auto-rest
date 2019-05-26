import { Component, OnInit, Inject, Output } from '@angular/core';
import { ActivatedRoute, Router } from '@angular/router';
import { GenericApi, OmAdmin } from '../rest.service';
import { Subject } from 'rxjs';

@Component({
    selector: 'app-om_admin-add',
    template: `
<form (submit)="submit({name: name.value, mail: mail.value, sm_login: sm_login.value})">
    <label for="name">name</label><input #name type="text" name="name" id="name"><br>
    <label for="mail">mail</label><input #mail type="text" name="mail" id="mail"><br>
    <label for="sm_login">sm_login</label><input #sm_login type="text" name="sm_login" id="sm_login"><br>
    <button type="submit">save</button>
</form>`,
})
export class OmAdminAddComponent implements OnInit {

    @Output() done = new Subject();


    constructor(
        @Inject("OmAdminService") private rest: GenericApi<OmAdmin>,
        private route: ActivatedRoute,
        private router: Router,
    ) { }

    ngOnInit() {

    }

    submit(data: OmAdmin) {
        this.rest.add(data)
            .subscribe(_ => this.done.next(), console.error);
    }
}
