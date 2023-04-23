import { ComponentFixture, TestBed } from '@angular/core/testing';

import { TypstAngularComponent } from './typst.angular.component';

describe('TypstAngularComponent', () => {
  let component: TypstAngularComponent;
  let fixture: ComponentFixture<TypstAngularComponent>;

  beforeEach(async () => {
    await TestBed.configureTestingModule({
      declarations: [TypstAngularComponent],
    }).compileComponents();

    fixture = TestBed.createComponent(TypstAngularComponent);
    component = fixture.componentInstance;
    fixture.detectChanges();
  });

  it('should create', () => {
    expect(component).toBeTruthy();
  });
});
