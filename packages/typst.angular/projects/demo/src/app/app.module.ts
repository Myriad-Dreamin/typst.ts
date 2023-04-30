import { NgModule } from '@angular/core';
import { BrowserModule } from '@angular/platform-browser';
import { HttpClientModule } from '@angular/common/http';

import { AppRoutingModule } from './app-routing.module';
import { AppComponent } from './app.component';

import { TypstDocumentModule } from '@myriaddreamin/typst.angular';
import { AppService } from './app.service';

@NgModule({
  declarations: [AppComponent],
  imports: [BrowserModule, HttpClientModule, AppRoutingModule, TypstDocumentModule],
  providers: [AppService],
  bootstrap: [AppComponent],
})
export class AppModule {}
