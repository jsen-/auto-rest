import { Component, OnInit } from '@angular/core';
import { RestService, Product } from '../rest.service';
import { ActivatedRoute, Router } from '@angular/router';

@Component({
    selector: 'app-om_server',
    template: `
    <h2>Product List</h2>

    <div>
      <button (click)="add()">
        Add
      </button>
    </div>

    <ul class="products">
      <li *ngFor="let p of products; let i=index;">
        <a routerLink="/product-details/{{p._id}}">
          <span class="badge">{{i+1}}</span> {{p.prod_name}}
        </a>
        <button class="delete" title="delete product"
          (click)="delete(p._id)">x</button>
      </li>
    </ul>`,
    styleUrls: ['./om_server.component.css']
})
export class OmServerComponent implements OnInit {

    products: any = [];

    constructor(public rest: RestService, private route: ActivatedRoute, private router: Router) { }

    ngOnInit() {
        this.getProducts();
    }

    getProducts() {
        this.products = [];
        this.rest.product.get_all().subscribe((data: {}) => {
            console.log(data);
            this.products = data;
        });
    }

    add() {
        this.router.navigate(['/product-add']);
    }

    delete(id) {
        this.rest.product.delete(id)
            .subscribe(res => {
                this.getProducts();
            }, (err) => {
                console.log(err);
            });
    }

}
