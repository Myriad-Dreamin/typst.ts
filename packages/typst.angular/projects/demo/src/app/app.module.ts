import { NgModule } from '@angular/core';
import { BrowserModule } from '@angular/platform-browser';

import { AppRoutingModule } from './app-routing.module';
import { AppComponent } from './app.component';

import { TypstDocumentModule } from '@myriaddreamin/typst.angular';

@NgModule({
  declarations: [AppComponent],
  imports: [BrowserModule, AppRoutingModule, TypstDocumentModule],
  providers: [],
  bootstrap: [AppComponent],
})
export class AppModule {}
