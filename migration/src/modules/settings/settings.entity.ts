import { Entity, PrimaryKey, Property } from '@mikro-orm/core';

@Entity()
export class Settings {
  @PrimaryKey({ type: 'text' })
  key!: string

  @Property({ type: 'text' })
  value!: string
}