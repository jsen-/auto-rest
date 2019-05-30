import { Component, OnInit, Inject, Output } from '@angular/core';
import { ActivatedRoute, Router } from '@angular/router';
import { GenericApi, OmAdmin, SansId } from '../rest.service';
import { Subject } from 'rxjs';

@Component({
    selector: 'app-om_admin-add',
    styles: [`
#container {
    display: inline-grid;
    grid-template-columns: auto 1fr;
    grid-template-rows: 20% 20% 20% 20%;
}
#container label {
    grid-column-start: 1;
}
#container input {
    grid-column-start: 2;
}
#container button {
    grid-column-start: span 2;
}
`],
    template: `
<form id="container" (submit)="submit({name: name.value, mail: mail.value, sm_login: sm_login.value})">
    <label for="name">name</label><input #name type="text" name="name" id="name">
    <label for="mail">mail</label><input #mail type="text" name="mail" id="mail">
    <label for="sm_login">sm_login</label><input #sm_login type="text" name="sm_login" id="sm_login">
    <button type="submit">add</button>
</form>`,
})
export class OmAdminAddComponent {

    @Output() done = new Subject();
    @Output() error = new Subject<string>();
    f = {};

    constructor(@Inject("OmAdminService") private rest: GenericApi<OmAdmin>) { }

    visualize_error(err: any) {
        if (err.error) {
            let e = err.error;
            if (e.error && e.field) {
// ##
            }
        }
    }

    submit(data: SansId<OmAdmin>) {
        this.rest.add(data)
            .subscribe(
                _ => this.done.next(),
                err => this.error.next(err.error ? err.error.error : err)
            );
    }
}
