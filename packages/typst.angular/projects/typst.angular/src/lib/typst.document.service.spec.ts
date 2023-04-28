import { TestBed } from '@angular/core/testing';

import { TypstDocumentService } from './typst.document.service';

describe('TypstDocumentService', () => {
  let service: TypstDocumentService;

  beforeEach(() => {
    TestBed.configureTestingModule({});
    service = TestBed.inject(TypstDocumentService);
  });

  it('should be created', () => {
    expect(service).toBeTruthy();
  });
});
