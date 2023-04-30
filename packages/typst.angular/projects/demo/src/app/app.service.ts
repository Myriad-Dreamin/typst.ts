import { Injectable } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { Observable } from 'rxjs';

@Injectable({
  providedIn: 'root',
})
export class AppService {
  constructor(private http: HttpClient) {}

  getArtifact(): Observable<string> {
    return this.http.get('http://localhost:20810/hw/main.artifact.json', {
      responseType: 'text',
    });
  }
}
