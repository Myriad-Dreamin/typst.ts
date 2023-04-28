import { ComponentFixture, TestBed } from '@angular/core/testing';

import { TypstDocumentComponent } from './typst.document.component';

describe('TypstDocumentComponent', () => {
  let component: TypstDocumentComponent;
  let fixture: ComponentFixture<TypstDocumentComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      declarations: [TypstDocumentComponent],
    }).compileComponents();

    fixture = TestBed.createComponent(TypstDocumentComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
