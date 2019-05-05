import { Component, OnInit, Input } from '@angular/core';
import { RestService, Product } from '../rest.service';
import { ActivatedRoute, Router } from '@angular/router';

@Component({
    selector: 'app-product-add',
    templateUrl: './product-add.component.html',
    styleUrls: ['./product-add.component.css']
})
export class ProductAddComponent {
    @Input() data = Product.new();

    constructor(public rest: RestService, private route: ActivatedRoute, private router: Router) { }

    add() {
        this.rest.product.add(this.data)
            .subscribe((result) => {
                this.router.navigate([`/product-details/${result.id}`]);
            }, (err) => {
                console.log(err);
            });
    }

}
