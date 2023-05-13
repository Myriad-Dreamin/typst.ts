import { Injectable } from '@angular/core';
import { HttpClient } from '@angular/common/http';
import { Observable, map } from 'rxjs';

@Injectable({
  providedIn: 'root',
})
export class AppService {
  constructor(private http: HttpClient) {}

  getArtifact(): Observable<Uint8Array> {
    return this.http
      .get('http://localhost:20810/skyzh-cv/main.artifact.json', {
        responseType: 'arraybuffer',
      })
      .pipe(map(a => new Uint8Array(a)));
  }
}
