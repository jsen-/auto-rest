import { Component, OnInit } from '@angular/core';
import { RestService, Product } from '../rest.service';
import { ActivatedRoute, Router } from '@angular/router';

@Component({
    selector: 'app-product-detail',
    templateUrl: './product-detail.component.html',
    styleUrls: ['./product-detail.component.css']
})
export class ProductDetailComponent implements OnInit {

    product: Product;

    constructor(public rest: RestService, private route: ActivatedRoute, private router: Router) { }

    ngOnInit() {
        this.rest.product.get(this.route.snapshot.params['id'])
            .subscribe((data: Product) => {
                console.log(data);
                this.product = data;
            });
    }

}
