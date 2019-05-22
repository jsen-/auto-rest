import { Component, OnInit, Input } from '@angular/core';
import { RestService, Product } from '../rest.service';
import { ActivatedRoute, Router } from '@angular/router';

@Component({
    selector: 'app-product-add',
    template: `
    <div>
    <h2>Product Add</h2>
    <div>
        <label>Name:
            <input [(ngModel)]="data.name" placeholder="Name"/>
        </label><br>
        <label>Description:
            <input [(ngModel)]="data.desc" placeholder="Description"/>
        </label><br>
        <label>Price:
            <input [(ngModel)]="data.price" placeholder="Price"/>
        </label><br>
    </div>
    <button (click)="save()">Save</button>
  </div>`,
    styleUrls: ['./product-add.component.css']
})
export class ProductAddComponent {
    @Input() data: Product = {} as any;

    constructor(public rest: RestService, private route: ActivatedRoute, private router: Router) { }

    save() {
        this.rest.product.add(this.data)
            .subscribe((result) => {
                this.router.navigate([`/product-details/${result.id}`]);
            }, (err) => {
                console.log(err);
            });
    }

}
