import { BrowserModule } from "@angular/platform-browser";
import { NgModule } from "@angular/core";
import { HttpClientModule, HttpClient } from "@angular/common/http";
import { RouterModule, Routes } from "@angular/router";
import { FormsModule } from "@angular/forms";

import { AppComponent } from "./app.component";

import { OmServerListComponent } from "./om_server/om_server.component";
import { OmAdminListComponent } from "./om_admin/om_admin-list.component";
import { OmEnvironmentListComponent } from "./om_environment/om_environment.component";
import { OmAdmin, GenericApi, OmEnvironment, OmServer } from "./rest.service";

const endpoint = `${window.location.protocol}//${window.location.host}/api/v1/`;
const appRoutes: Routes = [
    {
        path: "om_server",
        component: OmServerListComponent,
        data: { title: "OM servers" }
    },
    {
        path: "om_environment",
        component: OmEnvironmentListComponent,
        data: { title: "OM environments" }
    },
    {
        path: "om_admin",
        component: OmAdminListComponent,
        data: { title: "OM admins" }
    },
    {
        path: "",
        redirectTo: "/om_admin",
        pathMatch: "full"
    }
];

@NgModule({
    declarations: [
        AppComponent,
        OmServerListComponent,
        OmEnvironmentListComponent,
        OmAdminListComponent,
    ],
    imports: [
        RouterModule.forRoot(appRoutes),
        FormsModule,
        BrowserModule,
        HttpClientModule
    ],
    providers: [
        {
            provide: "OmAdminService", deps: [HttpClient], useFactory: (httpClient: HttpClient) => {
                return new GenericApi<OmAdmin>(httpClient, `${endpoint}/om_admin`, "om_admin");
            }
        },
        {
            provide: "OmEnvironmentService", deps: [HttpClient], useFactory: (httpClient: HttpClient) => {
                return new GenericApi<OmEnvironment>(httpClient, `${endpoint}/om_environment`, "om_environment");
            }
        },
        {
            provide: "OmServerService", deps: [HttpClient], useFactory: (httpClient: HttpClient) => {
                return new GenericApi<OmServer>(httpClient, `${endpoint}/om_server`, "om_server");
            }
        },
    ],
    bootstrap: [AppComponent]
})
export class AppModule { }
