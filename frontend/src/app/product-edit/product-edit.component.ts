import { Component, OnInit, Input } from "@angular/core";
import { RestService, Product } from "../rest.service";
import { ActivatedRoute, Router } from "@angular/router";

@Component({
    selector: "app-product-edit",
    templateUrl: "./product-edit.component.html",
    styleUrls: ["./product-edit.component.css"]
})
export class ProductEditComponent implements OnInit {

    @Input() data = {} as Product;

    constructor(public rest: RestService, private route: ActivatedRoute, private router: Router) { }

    ngOnInit() {
        this.rest.product.get(this.route.snapshot.params["id"])
            .subscribe((data: Product) => {
                console.log(data);
                this.data = data;
            });
    }

    update() {
        this.rest.product.update(this.route.snapshot.params["id"], this.data)
            .subscribe((result) => {
                this.router.navigate([`/product-details/${result.id}`]);
            }, (err) => {
                console.log(err);
            });
    }

}
