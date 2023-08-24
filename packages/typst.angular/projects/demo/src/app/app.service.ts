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
      .get('http://localhost:20810/corpus/skyzh-cv/main.white.artifact.json', {
        responseType: 'arraybuffer',
      })
      .pipe(map(a => new Uint8Array(a)));
  }
}
