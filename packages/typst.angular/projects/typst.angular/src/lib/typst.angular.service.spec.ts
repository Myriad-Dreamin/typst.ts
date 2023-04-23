import { TestBed } from '@angular/core/testing';

import { TypstAngularService } from './typst.angular.service';

describe('TypstAngularService', () => {
  let service: TypstAngularService;

  beforeEach(() => {
    TestBed.configureTestingModule({});
    service = TestBed.inject(TypstAngularService);
  });

  it('should be created', () => {
    expect(service).toBeTruthy();
  });
});
